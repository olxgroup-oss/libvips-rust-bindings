use inflector::Inflector;
use std::env;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Debug, Clone)]
struct Operation {
    name: String,
    vips_name: String,
    vips_operation: String,
    description: String,
    required: Vec<Parameter>,
    optional: Vec<Parameter>,
    output: Vec<Parameter>,
}

impl Operation {
    fn doc_base(&self) -> String {
        let mut dc = format!("/// {}\n", self.description);
        let required = self
            .required
            .iter()
            .map(|r| r.doc())
            .collect::<Vec<_>>()
            .join("\n");
        dc.push_str(required.as_str());
        dc
    }

    fn doc_optional(&self) -> String {
        format!(
            "/// {}_options: `&{}Options` -> optional arguments",
            self.name,
            self.name.to_class_case()
        )
    }

    fn doc_returns(&self) -> String {
        if self.output.len() == 1 {
            format!(
                "/// returns `{}` - {}",
                self.output[0].param_type.struct_type(),
                self.output[0].description
            )
        } else if self.output.len() > 1 {
            let res = self
                .output
                .iter()
                .map(|o| format!("/// {} - {}", o.param_type.struct_type(), o.description))
                .collect::<Vec<_>>()
                .join("\n");
            format!("/// Tuple (\n{}\n///)", res)
        } else {
            String::new()
        }
    }

    fn doc(&self, with_optional: bool) -> String {
        let base = self.doc_base();
        let returns = self.doc_returns();
        if self.optional.len() > 0 && with_optional {
            format!("{}\n{}\n{}", base, self.doc_optional(), returns)
        } else {
            format!("{}\n{}", base, returns)
        }
    }

    fn struct_options(&self) -> String {
        let declarations = self
            .optional
            .iter()
            .map(|p| format!("{}\npub {}", p.doc_struct(), p.struct_declaration()))
            .collect::<Vec<_>>()
            .join(",\n");
        let defaults = self
            .optional
            .iter()
            .map(|p| p.default())
            .collect::<Vec<_>>()
            .join(",\n");
        format!(
            r#"
            /// Options for {} operation
            #[derive(Clone, Debug)]
            pub struct {}Options {{
                {}
            }}

            impl std::default::Default for {}Options {{
                fn default() -> Self {{
                    {}Options {{
                        {}
                    }}
                }}
            }}
            "#,
            self.name,
            self.name.to_class_case(),
            declarations,
            self.name.to_class_case(),
            self.name.to_class_case(),
            defaults
        )
    }

    fn get_variables(&self, with_optional: bool) -> String {
        let in_declaration = self
            .required
            .iter()
            .map(|p| p.declare_in_variable())
            .collect::<Vec<_>>()
            .join("\n");
        let out_declaration = self
            .output
            .iter()
            .map(|p| p.declare_out_variable())
            .collect::<Vec<_>>()
            .join("\n");
        let opt_declaration = if with_optional {
            self.optional
                .iter()
                .map(|p| {
                    format!(
                        r#"
            {}
            {}"#,
                        p.declare_in_variable_optional(&self.name.to_snake_case()),
                        p.declare_opt_name()
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            String::new()
        };
        format!(
            r#"
        {}
        {}
        {}
        "#,
            in_declaration, out_declaration, opt_declaration
        )
    }

    fn get_params(&self, with_optonal: bool) -> String {
        let mut all_params = Vec::new();
        all_params.append(&mut self.required.clone());
        all_params.append(&mut self.output.clone());
        all_params.sort_by_key(|p| p.order);
        let params = all_params
            .iter()
            .map(|p| {
                if self.output.contains(p) {
                    match p.param_type.clone() {
                        ParamType::ArrayByte => {
                            format!("&mut {}_out, &mut {}_buf_size", p.name, p.name)
                        }
                        ParamType::ArrayInt | ParamType::ArrayDouble | ParamType::ArrayImage => {
                            format!("&mut {}_out, &mut {}_array_size", p.name, p.name)
                        }
                        ParamType::VipsImage { prev: Some(prev) } => {
                            format!("&mut {}_out, {}.len() as i32", p.name, prev)
                        }
                        _ => format!("&mut {}_out", p.name),
                    }
                } else {
                    match p.param_type {
                        ParamType::ArrayInt | ParamType::ArrayDouble => {
                            format!("{}_in, {}.len() as i32", p.name, p.name)
                        }
                        ParamType::ArrayImage => format!("{}_in", p.name),
                        ParamType::ArrayByte => format!("{}_in, {}.len() as u64", p.name, p.name),
                        ParamType::Enum { .. } => format!("{}_in.try_into().unwrap()", p.name),
                        ParamType::Str => format!("{}_in.as_ptr()", p.name),
                        _ => format!("{}_in", p.name),
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        if with_optonal {
            format!(
                "{},{}",
                params,
                self.optional
                    .iter()
                    .map(|p| p.opt_param_pair())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            params
        }
    }

    fn method_body(&self, with_optional: bool) -> String {
        let out_tuple = self
            .output
            .iter()
            .map(|p| p.as_out_param())
            .collect::<Vec<_>>()
            .join(",");
        let out_result = if self.output.len() > 1 {
            format!("({})", out_tuple)
        } else if self.output.is_empty() {
            String::from("()")
        } else {
            out_tuple
        };
        format!(
            r#"
        unsafe {{
            {}
            let vips_op_response = bindings::vips_{}({}, NULL);
            utils::result(vips_op_response, {}, Error::{}Error)
        }}
        "#,
            self.get_variables(with_optional),
            self.vips_name,
            self.get_params(with_optional),
            out_result,
            self.name.to_class_case()
        )
    }

    fn declaration(&self, with_optional: bool) -> String {
        let name = if with_optional {
            format!("{}_with_opts", self.name)
        } else {
            self.name.clone()
        };
        let params = if with_optional {
            let opt = format!(
                "{}_options: &{}Options",
                self.name.to_snake_case(),
                self.name.to_class_case()
            );
            let params = self
                .required
                .iter()
                .map(|p| p.param_declaration())
                .collect::<Vec<_>>()
                .join(", ");
            if params.is_empty() {
                opt
            } else {
                format!("{}, {}", params, opt)
            }
        } else {
            self.required
                .iter()
                .map(|p| p.param_declaration())
                .collect::<Vec<_>>()
                .join(", ")
        };
        let return_type = if self.output.len() == 0 {
            String::from("()")
        } else if self.output.len() == 1 {
            self.output[0].param_type.struct_type()
        } else {
            let types = self
                .output
                .iter()
                .map(|p| p.param_type.struct_type())
                .collect::<Vec<_>>()
                .join(", ");
            format!("({})", types)
        };
        format!("pub fn {}({}) -> Result<{}>", name, params, return_type)
    }

    fn enumeration(&self) -> Vec<String> {
        self.required
            .iter()
            .chain(self.optional.iter())
            .chain(self.output.iter())
            .map(|p| p.enumeration())
            .filter(|s| !s.is_empty())
            .collect::<Vec<String>>()
    }

    fn body(&self) -> String {
        let mut main = format!(
            r#"
        {}
        {} {{
            {}
        }}
        "#,
            self.doc(false),
            self.declaration(false),
            self.method_body(false)
        );
        if self.optional.len() > 0 {
            main.push_str(
                format!(
                    r#"
        {}
        {}
        {} {{
            {}
        }}
        "#,
                    self.struct_options(),
                    self.doc(true),
                    self.declaration(true),
                    self.method_body(true)
                )
                .as_str(),
            );
        }
        main
    }
}

#[derive(Debug, Clone)]
struct Parameter {
    order: u8,
    name: String,
    vips_name: String,
    nick: String,
    description: String,
    param_type: ParamType,
}

impl PartialEq for Parameter {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Parameter {
    fn enumeration(&self) -> String {
        self.param_type.enumeration()
    }

    fn as_out_param(&self) -> String {
        match self.param_type {
            ParamType::ArrayByte => format!(
                "utils::new_byte_array({}_out, {}_buf_size)",
                self.name, self.name
            ),
            ParamType::VipsImage { .. } => format!("VipsImage{{ ctx: {}_out }}", self.name),
            ParamType::VipsInterpolate => format!("VipsInterpolate{{ ctx: {}_out }}", self.name),
            ParamType::VipsBlob => format!("VipsBlob{{ ctx: {}_out }}.into()", self.name),
            ParamType::Int { .. } | ParamType::UInt { .. } | ParamType::Double { .. } => {
                format!("{}_out", self.name)
            }
            ParamType::Bool { .. } => format!("{}_out != 0", self.name),
            ParamType::ArrayInt => format!(
                "utils::new_int_array({}_out, {}_array_size)",
                self.name, self.name
            ),
            ParamType::ArrayDouble => format!(
                "utils::new_double_array({}_out, {}_array_size)",
                self.name, self.name
            ),
            _ => format!("*{}_out", self.name),
        }
    }

    fn doc(&self) -> String {
        let mut main_doc = format!(
            "/// {}: `{}` -> {}",
            self.name,
            self.param_type.param_type(),
            self.description
        );
        let dc = self.param_type.doc();
        if !dc.is_empty() {
            main_doc.push_str("\n");
            main_doc.push_str(&dc);
        }
        main_doc
    }

    fn doc_struct(&self) -> String {
        let mut main_doc = format!(
            "/// {}: `{}` -> {}",
            self.name,
            self.param_type.struct_type(),
            self.description
        );
        let dc = self.param_type.doc();
        if !dc.is_empty() {
            main_doc.push_str("\n");
            main_doc.push_str(&dc);
        }
        main_doc
    }

    fn declare_in_variable(&self) -> String {
        match self.param_type {
            ParamType::Int { .. } => format!(
                "let {}_in: {} = {};",
                self.name,
                self.param_type.vips_in_type(false),
                self.name
            ),
            ParamType::UInt { .. } => format!(
                "let {}_in: {} = {};",
                self.name,
                self.param_type.vips_in_type(false),
                self.name
            ),
            ParamType::Double { .. } => format!(
                "let {}_in: {} = {};",
                self.name,
                self.param_type.vips_in_type(false),
                self.name
            ),
            ParamType::Str => format!(
                "let {}_in: {} = utils::new_c_string({})?;",
                self.name,
                self.param_type.vips_in_type(false),
                self.name
            ),
            ParamType::Bool { .. } => format!(
                "let {}_in: {} = if {} {{ 1 }} else {{ 0 }};",
                self.name,
                self.param_type.vips_in_type(false),
                self.name
            ),
            ParamType::ArrayInt => format!(
                "let {}_in: {} = {}.as_mut_ptr();",
                self.name,
                self.param_type.vips_in_type(false),
                self.name
            ),
            ParamType::ArrayDouble => format!(
                "let {}_in: {} = {}.as_mut_ptr();",
                self.name,
                self.param_type.vips_in_type(false),
                self.name
            ),
            ParamType::ArrayByte => format!(
                "let {}_in: {} = {}.as_ptr() as {};",
                self.name,
                self.param_type.vips_in_type(false),
                self.name,
                self.param_type.vips_in_type(false)
            ),
            ParamType::ArrayImage => format!(
                "let {}_in: {} = {}.iter().map(|v| v.ctx).collect::<Vec<_>>().as_mut_ptr();",
                self.name,
                self.param_type.vips_in_type(false),
                self.name
            ),
            ParamType::VipsInterpolate => format!(
                "let {}_in: {} = {}.ctx;",
                self.name,
                self.param_type.vips_in_type(false),
                self.name
            ),
            ParamType::VipsImage { .. } => format!(
                "let {}_in: {} = {}.ctx;",
                self.name,
                self.param_type.vips_in_type(false),
                self.name
            ),
            ParamType::VipsBlob => format!(
                "let {}_in: {} = {}.ctx;",
                self.name,
                self.param_type.vips_in_type(false),
                self.name
            ),
            ParamType::Enum { .. } => format!(
                "let {}_in: {} = {} as i32;",
                self.name,
                self.param_type.vips_in_type(false),
                self.name
            ),
        }
    }

    fn declare_in_variable_optional(&self, opt_name: &str) -> String {
        match self.param_type {
            ParamType::Int { .. } | ParamType::UInt { .. } | ParamType::Double { .. } => format!(
                "let {}_in: {} = {}_options.{};",
                self.name,
                self.param_type.vips_in_type(true),
                opt_name,
                self.name
            ),
            ParamType::Str => format!(
                "let {}_in: {} = utils::new_c_string(&{}_options.{})?;",
                self.name,
                self.param_type.vips_in_type(true),
                opt_name,
                self.name
            ),
            ParamType::Bool { .. } => format!(
                "let {}_in: {} = if {}_options.{} {{ 1 }} else {{ 0 }};",
                self.name,
                self.param_type.vips_in_type(true),
                opt_name,
                self.name
            ),
            ParamType::ArrayDouble | ParamType::ArrayImage | ParamType::ArrayInt => format!(
                "let {}_wrapper = {}::from(&{}_options.{}[..]); \nlet {}_in = {}_wrapper.ctx;",
                self.name,
                self.param_type.vips_in_type(true),
                opt_name,
                self.name,
                self.name,
                self.name
            ),
            ParamType::ArrayByte => format!(
                "let {}_in: {} = {}_options.{}.as_mut_ptr();",
                self.name,
                self.param_type.vips_in_type(true),
                opt_name,
                self.name
            ),
            ParamType::VipsBlob | ParamType::VipsImage { .. } | ParamType::VipsInterpolate => {
                format!(
                    "let {}_in: {} = {}_options.{}.ctx;",
                    self.name,
                    self.param_type.vips_in_type(true),
                    opt_name,
                    self.name
                )
            }
            ParamType::Enum { .. } => format!(
                "let {}_in: {} = {}_options.{} as i32;",
                self.name,
                self.param_type.vips_in_type(true),
                opt_name,
                self.name
            ),
        }
    }

    fn declare_opt_name(&self) -> String {
        format!(
            "let {}_in_name = utils::new_c_string(\"{}\")?;",
            self.name, self.vips_name
        )
    }

    fn opt_param_pair(&self) -> String {
        let init_var = match self.param_type {
            ParamType::Str => format!("{}_in.as_ptr()", self.name),
            _ => format!("{}_in", self.name),
        };
        format!("{}_in_name.as_ptr(), {}", self.name, init_var)
    }

    fn declare_out_variable(&self) -> String {
        match self.param_type {
            ParamType::ArrayByte { .. } => format!(
                "let mut {}_buf_size: u64 = 0;\nlet mut {}_out: {} = null_mut();",
                self.name,
                self.name,
                self.param_type.vips_out_type()
            ),
            ParamType::Int { .. } | ParamType::Double { .. } | ParamType::UInt { .. } => format!(
                "let mut {}_out: {} = {};",
                self.name,
                self.param_type.vips_out_type(),
                self.param_type.default()
            ),
            ParamType::ArrayDouble | ParamType::ArrayInt | ParamType::ArrayImage => format!(
                "let mut {}_array_size: usize = 0;\nlet mut {}_out: {} = null_mut();",
                self.name,
                self.name,
                self.param_type.vips_out_type()
            ),
            ParamType::Bool { .. } => format!(
                "let mut {}_out: {} = 0;",
                self.name,
                self.param_type.vips_out_type()
            ),
            _ => format!(
                "let mut {}_out: {} = null_mut();",
                self.name,
                self.param_type.vips_out_type()
            ),
        }
    }

    fn default(&self) -> String {
        match self.param_type {
            ParamType::Str if self.description.contains("ICC") => {
                format!("{}: String::from(\"sRGB\")", self.name)
            }
            _ => format!("{}: {}", self.name, self.param_type.default()),
        }
    }

    fn struct_declaration(&self) -> String {
        format!("{}: {}", self.name, self.param_type.struct_type())
    }

    fn param_declaration(&self) -> String {
        format!("{}: {}", self.name, self.param_type.param_type())
    }
}

#[derive(Debug, Clone)]
enum ParamType {
    Int {
        min: i32,
        max: i32,
        default: i32,
    },
    UInt {
        min: u64,
        max: u64,
        default: u64,
    },
    Double {
        min: f64,
        max: f64,
        default: f64,
    },
    Str,
    Enum {
        name: String,
        entries: Vec<Enumeration>,
        default: i32,
    },
    Bool {
        default: bool,
    },
    ArrayInt,
    ArrayDouble,
    ArrayImage,
    ArrayByte,
    VipsInterpolate,
    VipsImage {
        prev: Option<String>,
    },
    VipsBlob,
}

impl ParamType {
    fn doc(&self) -> String {
        match self {
            ParamType::Int { min, max, default } => {
                format!("/// min: {}, max: {}, default: {}", min, max, default)
            }
            ParamType::UInt { min, max, default } => {
                format!("/// min: {}, max: {}, default: {}", min, max, default)
            }
            ParamType::Double { min, max, default } => {
                format!("/// min: {}, max: {}, default: {}", min, max, default)
            }
            ParamType::Bool { default } => format!("/// default: {}", default),
            ParamType::Enum {
                entries, default, ..
            } => entries
                .into_iter()
                .map(|e| {
                    if *default == e.value {
                        format!("{} [DEFAULT]", e.doc())
                    } else {
                        format!("{}", e.doc())
                    }
                })
                .collect::<Vec<_>>()
                .join("\n"),
            _ => String::new(),
        }
    }

    fn struct_type(&self) -> String {
        match self {
            ParamType::Int { .. } => String::from("i32"),
            ParamType::UInt { .. } => String::from("u64"),
            ParamType::Double { .. } => String::from("f64"),
            ParamType::Str => String::from("String"),
            ParamType::Bool { .. } => String::from("bool"),
            ParamType::ArrayInt => String::from("Vec<i32>"),
            ParamType::ArrayDouble => String::from("Vec<f64>"),
            ParamType::ArrayByte => String::from("Vec<u8>"),
            ParamType::ArrayImage => String::from("Vec<VipsImage>"),
            ParamType::VipsInterpolate => String::from("VipsInterpolate"),
            ParamType::VipsImage { .. } => String::from("VipsImage"),
            ParamType::VipsBlob => String::from("Vec<u8>"),
            ParamType::Enum { name, .. } => Self::enum_name(name),
        }
    }

    fn param_type(&self) -> String {
        match self {
            ParamType::Int { .. } => String::from("i32"),
            ParamType::UInt { .. } => String::from("u64"),
            ParamType::Double { .. } => String::from("f64"),
            ParamType::Str => String::from("&str"),
            ParamType::Bool { .. } => String::from("bool"),
            ParamType::ArrayInt => String::from("&mut [i32]"),
            ParamType::ArrayDouble => String::from("&mut [f64]"),
            ParamType::ArrayByte => String::from("&[u8]"),
            ParamType::ArrayImage => String::from("&mut [VipsImage]"),
            ParamType::VipsInterpolate => String::from("&VipsInterpolate"),
            ParamType::VipsImage { .. } => String::from("&VipsImage"),
            ParamType::VipsBlob => String::from("&[u8]"),
            ParamType::Enum { name, .. } => Self::enum_name(name),
        }
    }

    fn enum_name(name: &str) -> String {
        let split: Vec<&str> = name.split("Vips").collect();
        if split.len() > 1 {
            format!("{}", split[1])
        } else {
            format!("{}", split[0])
        }
    }

    fn vips_in_type(&self, is_optional: bool) -> String {
        match self {
            ParamType::Int { .. } => String::from("i32"),
            ParamType::UInt { .. } => String::from("u64"),
            ParamType::Double { .. } => String::from("f64"),
            ParamType::Str => String::from("CString"),
            ParamType::Bool { .. } => String::from("i32"),
            ParamType::ArrayInt => {
                if !is_optional {
                    String::from("*mut i32")
                } else {
                    String::from("utils::VipsArrayIntWrapper")
                }
            }
            ParamType::ArrayDouble => {
                if !is_optional {
                    String::from("*mut f64")
                } else {
                    String::from("utils::VipsArrayDoubleWrapper")
                }
            }
            ParamType::ArrayByte => String::from("*mut c_void"),
            ParamType::ArrayImage => {
                if !is_optional {
                    String::from("*mut *mut bindings::VipsImage")
                } else {
                    String::from("utils::VipsArrayImageWrapper")
                }
            }
            ParamType::VipsInterpolate => String::from("*mut bindings::VipsInterpolate"),
            ParamType::VipsImage { .. } => String::from("*mut bindings::VipsImage"),
            ParamType::VipsBlob => String::from("*mut bindings::VipsBlob"),
            ParamType::Enum { .. } => String::from("i32"),
        }
    }

    fn vips_out_type(&self) -> String {
        match self {
            ParamType::Int { .. } => String::from("i32"),
            ParamType::UInt { .. } => String::from("u64"),
            ParamType::Double { .. } => String::from("f64"),
            ParamType::Str => String::from("*mut c_char"),
            ParamType::Bool { .. } => String::from("i32"),
            ParamType::ArrayInt => String::from("*mut i32"),
            ParamType::ArrayDouble => String::from("*mut f64"),
            ParamType::ArrayByte => String::from("*mut c_void"),
            ParamType::ArrayImage => String::from("*mut bindings::VipsImage"),
            ParamType::VipsInterpolate => String::from("*mut bindings::VipsInterpolate"),
            ParamType::VipsImage { .. } => String::from("*mut bindings::VipsImage"),
            ParamType::VipsBlob => String::from("*mut bindings::VipsBlob"),
            ParamType::Enum { .. } => String::from("*mut i32"),
        }
    }

    fn default(&self) -> String {
        match self {
            ParamType::Int { default, .. } => format!("i32::from({})", default.to_string()),
            ParamType::UInt { default, .. } => default.to_string(),
            ParamType::Double { default, .. } => format!("f64::from({})", default.to_string()),
            ParamType::Str => String::from("String::new()"),
            ParamType::Bool { default, .. } => default.to_string(),
            ParamType::ArrayInt => String::from("Vec::new()"),
            ParamType::ArrayDouble => String::from("Vec::new()"),
            ParamType::ArrayByte => String::from("Vec::new()"),
            ParamType::ArrayImage => String::from("Vec::new()"),
            ParamType::VipsInterpolate => String::from("VipsInterpolate::new()"),
            ParamType::VipsImage { .. } => String::from("VipsImage::new()"),
            ParamType::VipsBlob => String::from("Vec::new()"),
            ParamType::Enum {
                name,
                entries,
                default,
            } => entries
                .iter()
                .filter(|e| *default == e.value)
                .map(|e| format!("{}::{}", Self::enum_name(name), e.nick.to_class_case()))
                .collect::<Vec<_>>()[0]
                .clone(),
        }
    }

    fn enumeration(&self) -> String {
        match self {
            ParamType::Enum { name, entries, .. } => {
                let enum_entries = entries
                    .iter()
                    .map(|e| e.code())
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    r#"
                #[derive(Copy, Clone, Debug, FromPrimitive)]
                pub enum {} {{
                    {}
                }}
                "#,
                    Self::enum_name(&name),
                    enum_entries
                )
            }
            _ => String::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct Enumeration {
    name: String,
    nick: String,
    value: i32,
}

impl Enumeration {
    fn doc(&self) -> String {
        format!(
            "///  `{}` -> {} = {}",
            self.nick.to_class_case(),
            self.name,
            self.value
        )
    }

    fn code(&self) -> String {
        format!(
            "{}\n{} = {},",
            self.doc(),
            if self.name == "VIPS_INTERPRETATION_LABS" {
                String::from("Labs")
            } else {
                self.nick.to_class_case()
            },
            self.value
        )
    }
}

fn split_flags(output: &[u8]) -> Vec<String> {
    let mut word = Vec::new();
    let mut words = Vec::new();
    let mut escaped = false;

    for &b in output {
        match b {
            _ if escaped => {
                escaped = false;
                word.push(b);
            }
            b'\\' => escaped = true,
            b' ' | b'\n' | b'\r' => {
                if !word.is_empty() {
                    words.push(String::from_utf8(word).unwrap());
                    word = Vec::new();
                }
            }
            _ => word.push(b),
        }
    }

    if !word.is_empty() {
        words.push(String::from_utf8(word).unwrap());
    }

    words
}

fn parse_param(param_list: Vec<&str>, order: u8, prev: Option<String>) -> (bool, Parameter) {
    let (mut param_name, is_output) = if param_list[0].starts_with("OUTPUT:") {
        let splited: Vec<&str> = param_list[0].split("OUTPUT:").collect();
        (String::from(splited[1]), true)
    } else {
        (String::from(param_list[0]), false)
    };
    if vec!["in", "ref"].contains(&param_name.as_str()) {
        param_name = format!("{}p", param_name);
    }
    let nick = param_list[1];
    let description = param_list[2];
    let param_type = if param_list[3].starts_with("string") {
        ParamType::Str
    } else if param_list[3].starts_with("VipsImage") {
        ParamType::VipsImage { prev }
    } else if param_list[3].starts_with("VipsBlob") {
        ParamType::VipsBlob
    } else if param_list[3].starts_with("VipsInterpolate") {
        ParamType::VipsInterpolate
    } else if param_list[3].starts_with("bool") {
        let default = param_list[3].split(':').collect::<Vec<&str>>()[1] == "1";
        ParamType::Bool { default }
    } else if param_list[3].starts_with("int") {
        let strs: Vec<&str> = param_list[3].split(':').collect();

        let min = strs[1].parse().expect("Cannot parse number");
        let max = strs[2].parse().expect("Cannot parse number");
        let default = strs[3].parse().expect("Cannot parse number");
        ParamType::Int { min, max, default }
    } else if param_list[3].starts_with("double") {
        let strs: Vec<&str> = param_list[3].split(':').collect();

        let min = strs[1].parse().expect("Cannot parse number");
        let max = strs[2].parse().expect("Cannot parse number");
        let default = strs[3].parse().expect("Cannot parse number");
        ParamType::Double { min, max, default }
    } else if param_list[3].starts_with("uint64") {
        let strs: Vec<&str> = param_list[3].split(':').collect();

        let min = strs[1].parse().expect("Cannot parse number");
        let max = strs[2].parse().expect("Cannot parse number");
        let default = strs[3].parse().expect("Cannot parse number");
        ParamType::UInt { min, max, default }
    } else if param_list[3].starts_with("byte-data") {
        ParamType::ArrayByte
    } else if param_list[3].starts_with("array of int") {
        ParamType::ArrayInt
    } else if param_list[3].starts_with("array of double") {
        ParamType::ArrayDouble
    } else if param_list[3].starts_with("array of images") {
        ParamType::ArrayImage
    } else if param_list[3].starts_with("enum") || param_list[3].starts_with("flags") {
        let enum_name = param_list[3].split("-").collect::<Vec<&str>>()[1];
        let mut enum_values = Vec::new();
        for i in 4..param_list.len() - 1 {
            let enum_strs: Vec<&str> = param_list[i].split(':').collect();
            let value = enum_strs[0].parse().expect("Cannot parse number");
            let nick = enum_strs[1].to_string();
            let name = enum_strs[2].to_string();
            enum_values.push(Enumeration { name, nick, value });
        }
        let default = param_list[param_list.len() - 1]
            .parse()
            .expect("Cannot parse number");
        ParamType::Enum {
            name: enum_name.to_string(),
            entries: enum_values,
            default: default,
        }
    } else {
        panic!("Unsupported type: {}", param_list[3])
    };
    (
        is_output,
        Parameter {
            order: order,
            name: param_name.to_snake_case(),
            vips_name: param_name.to_string(),
            nick: nick.to_class_case(),
            description: description.to_string(),
            param_type: param_type,
        },
    )
}

fn parse_output(output: String) -> Vec<Operation> {
    output
        .split("OPERATION:")
        .filter(|op| *op != "")
        .map(|op_str: &str| {
            let mut required: Vec<Parameter> = Vec::new();
            let mut optional: Vec<Parameter> = Vec::new();
            let mut output: Vec<Parameter> = Vec::new();

            let mut op_iter = op_str.lines().filter(|op| *op != "");

            let op_vals: Vec<&str> = op_iter
                .by_ref()
                .take_while(|line| *line != "REQUIRED:")
                .collect();

            let name_split = op_vals[0].split(":").collect::<Vec<_>>();
            let description = op_vals[1].to_string();

            let mut required_vals = op_iter
                .by_ref()
                .take_while(|line| *line != "OPTIONAL:")
                .skip(1)
                .peekable(); // skip the first line PARAM:
            let mut order: u8 = 0;
            while required_vals.peek().is_some() {
                //VipsAffine is wrong in the introspection
                if name_split[1] == "VipsAffine" && order > 1 {
                    required.push(Parameter {
                        order: 2,
                        name: String::from("a"),
                        vips_name: String::from("a"),
                        nick: String::from("Transformation Matrix"),
                        description: String::from("Transformation Matrix coefficient"),
                        param_type: ParamType::Double {
                            min: -std::f64::INFINITY,
                            max: std::f64::INFINITY,
                            default: 0.0,
                        },
                    });
                    required.push(Parameter {
                        order: 3,
                        name: String::from("b"),
                        vips_name: String::from("b"),
                        nick: String::from("Transformation Matrix"),
                        description: String::from("Transformation Matrix coefficient"),
                        param_type: ParamType::Double {
                            min: -std::f64::INFINITY,
                            max: std::f64::INFINITY,
                            default: 0.0,
                        },
                    });
                    required.push(Parameter {
                        order: 4,
                        name: String::from("c"),
                        vips_name: String::from("c"),
                        nick: String::from("Transformation Matrix"),
                        description: String::from("Transformation Matrix coefficient"),
                        param_type: ParamType::Double {
                            min: -std::f64::INFINITY,
                            max: std::f64::INFINITY,
                            default: 0.0,
                        },
                    });
                    required.push(Parameter {
                        order: 5,
                        name: String::from("d"),
                        vips_name: String::from("d"),
                        nick: String::from("Transformation Matrix"),
                        description: String::from("Transformation Matrix coefficient"),
                        param_type: ParamType::Double {
                            min: -std::f64::INFINITY,
                            max: std::f64::INFINITY,
                            default: 0.0,
                        },
                    });
                    required_vals
                        .by_ref()
                        .take_while(|line| *line != "PARAM:")
                        .for_each(drop);
                } else {
                    let prev = if required.len() > 0 && order == 1 {
                        match required[0].param_type {
                            ParamType::ArrayImage => Some(required[0].name.clone()),
                            _ => None,
                        }
                    } else {
                        None
                    };
                    let (is_output, param) = parse_param(
                        required_vals
                            .by_ref()
                            .take_while(|line| *line != "PARAM:")
                            .collect(),
                        order,
                        prev,
                    );
                    if is_output {
                        output.push(param);
                    } else {
                        required.push(param);
                    }
                    order = order + 1;
                }
            }
            let mut optionals = op_iter.skip(1).peekable();
            while optionals.peek().is_some() {
                let param_list = optionals
                    .by_ref()
                    .take_while(|line| *line != "PARAM:")
                    .collect();

                let (_, param) = parse_param(param_list, 0, None);
                optional.push(param);
            }
            Operation {
                name: if name_split[0] == "match" {
                    String::from("matches")
                } else {
                    String::from(name_split[0]).to_snake_case()
                },
                vips_name: String::from(name_split[0]),
                vips_operation: String::from(name_split[1]),
                description,
                required,
                optional,
                output,
            }
        })
        .collect()
}

fn run(mut cmd: Command) -> Vec<String> {
    let output = cmd.output().expect("Couldn't run pkg-config");
    split_flags(&output.stdout[..])
}

fn rustfmt_path() -> io::Result<PathBuf> {
    if let Ok(rustfmt) = env::var("RUSTFMT") {
        return Ok(rustfmt.into());
    }
    match which::which("rustfmt") {
        Ok(p) => Ok(p),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("{}", e))),
    }
}

fn rustfmt_generated_strin(source: &str) -> io::Result<String> {
    let rustfmt = rustfmt_path()?;
    let mut cmd = Command::new(&*rustfmt);

    cmd.stdin(Stdio::piped()).stdout(Stdio::piped());

    let mut child = cmd.spawn()?;
    let mut child_stdin = child.stdin.take().unwrap();
    let mut child_stdout = child.stdout.take().unwrap();

    let source = source.to_owned();

    // Write to stdin in a new thread, so that we can read from stdout on this
    // thread. This keeps the child from blocking on writing to its stdout which
    // might block us from writing to its stdin.
    let stdin_handle = ::std::thread::spawn(move || {
        let _ = child_stdin.write_all(source.as_bytes());
        source
    });

    let mut output = vec![];
    io::copy(&mut child_stdout, &mut output)?;

    let status = child.wait()?;
    let source = stdin_handle.join().expect(
        "The thread writing to rustfmt's stdin doesn't do \
         anything that could panic",
    );

    match String::from_utf8(output) {
        Ok(bindings) => match status.code() {
            Some(0) => Ok(bindings),
            Some(2) => Err(io::Error::new(
                io::ErrorKind::Other,
                "Rustfmt parsing errors.".to_string(),
            )),
            Some(3) => {
                println!("Rustfmt could not format some lines.");
                Ok(bindings)
            }
            _ => Err(io::Error::new(
                io::ErrorKind::Other,
                "Internal rustfmt error".to_string(),
            )),
        },
        _ => Ok(source),
    }
}

fn main() {
    let operation_blacklist = vec![
        "VipsForeignSaveDzBuffer",
        "crop",
        "VipsLinear",
        "VipsGetpoint",
    ];

    println!("cargo:rustc-link-lib=vips");
    println!("cargo:rustc-link-lib=glib-2.0");
    println!("cargo:rustc-link-lib=gobject-2.0");
    println!("cargo:rerun-if-changed=vips.h");
    let mut cmd = Command::new("pkg-config");
    cmd.args(&["--cflags", "vips"]);
    let flags = run(cmd);
    let out_path = PathBuf::from(env::var("BINDINGS_DIR").unwrap());

    let mut generator = bindgen::Builder::default()
        .header("vips.h")
        .blacklist_type("max_align_t")
        .blacklist_item("FP_NAN")
        .blacklist_item("FP_INFINITE")
        .blacklist_item("FP_ZERO")
        .blacklist_item("FP_SUBNORMAL")
        .blacklist_item("FP_NORMAL")
        .constified_enum("*")
        .generate_comments(true)
        .impl_debug(true)
        .impl_partialeq(true)
        .derive_debug(true)
        .derive_eq(true)
        .rustfmt_bindings(true);
    for flag in flags.into_iter() {
        generator = generator.clang_arg(flag);
    }
    let bindings = generator.generate().expect("Unable to generate bindings");

    let mut cmd_introspect = Command::new("pkg-config");
    cmd_introspect.args(&["--cflags", "--libs", "vips"]);
    let instrospect_flags = run(cmd_introspect);

    let mut cc_builder = cc::Build::new();
    for flag in instrospect_flags.into_iter() {
        cc_builder.flag(&flag);
    }
    let mut cc_cmd = cc_builder
        .no_default_flags(true)
        .out_dir("./")
        .flag("-ointrospect")
        .flag("-g")
        .get_compiler()
        .to_command();
    let result = cc_cmd
        .arg("introspect.c")
        .status();
    if result.is_ok() && !result.unwrap().success() {
        let mut cmd = Command::new("./compile.sh");
        let res = cmd.status().expect("Couldn't compile introspect.c");
        if !res.success() {
            panic!("Failed to compile introspect.c");
        }
    }

    let vips_introspection = Command::new("./introspect")
        .output()
        .expect("Failed to run vips introspection");

    let output =
        String::from_utf8(vips_introspection.stdout).expect("Could not parse introspection output");
    let operations = parse_output(output);
    println!("{:#?}", operations);
    println!(
        "{:#?}",
        operations
            .iter()
            .filter(|o| o.output.len() > 1)
            .map(|o| o.name.clone())
            .collect::<Vec<_>>()
    );

    let (methods, errors, errors_display) = operations
        .iter()
        .filter(|o| !operation_blacklist.contains(&o.vips_operation.as_str()))
        .fold(
            (String::new(), String::new(), String::new()),
            |(mut methods, mut errors, mut errors_display), operation| {
                methods.push_str(operation.body().as_str());
                errors.push_str(format!("{}Error,\n", operation.name.to_class_case()).as_str());
                errors_display.push_str(
                    format!(
                        "Error::{}Error => write!(f, \"vips error: {}Error. Check error buffer for more details\"),\n",
                        operation.name.to_class_case(),
                        operation.name.to_class_case()
                    )
                    .as_str(),
                );
                (methods, errors, errors_display)
            },
        );

    let mut enums: Vec<String> = operations
        .iter()
        .map(|o| o.enumeration().into_iter())
        .flatten()
        .collect();
    enums.sort();
    enums.dedup(); // not working

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    let ops_content = format!(
        r#"
    use std::ffi::*;
    use std::ptr::null_mut;
    use std::convert::TryInto;
    use crate::bindings;
    use crate::utils;
    use crate::VipsImage;
    use crate::VipsInterpolate;
    use crate::VipsBlob;
    use crate::error::*;
    use crate::Result;

    const NULL: *const c_void = null_mut();
    {}
    {}
    "#,
        enums.join("\n"),
        methods
    );

    let errors_content = format!(
        r#"
    #[derive(Debug)]
    pub enum Error {{
        InitializationError(&'static str),
        IOError(&'static str),
        LinearError,
        GetpointError,
        {}
    }}

    impl std::fmt::Display for Error {{
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{
            match self {{
                Error::InitializationError(msg) => write!(f, "vips error: InitializationError - {{}}", msg),
                Error::IOError(msg) => write!(f, "vips error: IOError - {{}}", msg),
                Error::LinearError => write!(f, "vips error: LinearError. Check error buffer for more details"),
                Error::GetpointError => write!(f, "vips error: GetpointError. Check error buffer for more details"),
                {}
            }}
        }}
    }}

    impl std::error::Error for Error {{}}

    "#,
        errors, errors_display
    );

    let errors_formated = if let Ok(formated) = rustfmt_generated_strin(&errors_content) {
        formated
    } else {
        errors_content
    };
    let ops_formated = if let Ok(formated) = rustfmt_generated_strin(&ops_content) {
        formated
    } else {
        ops_content
    };

    let mut file_ops = File::create(out_path.join("ops.rs")).expect("Can't create file");
    file_ops
        .write_all(ops_formated.as_bytes())
        .expect("Can't write to file");
    let mut file_errs = File::create(out_path.join("error.rs")).expect("Can't create file");
    file_errs
        .write_all(errors_formated.as_bytes())
        .expect("Can't write to file");
}
