// (c) Copyright 2019-2023 OLX
/* vips8 introspection
 *
 * compile with:
 *
 * 	gcc -o introspect -g -Wall introspect.c `pkg-config vips --cflags --libs`
 *
 * This example needs vips 7.39 or later. 
 *
 * try:
 *
 * 	./introspect
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <locale.h>
#include <vips/vips.h>

static void
show_argument(GParamSpec * pspec, VipsArgumentClass * argument_class) {
    GType otype = G_PARAM_SPEC_VALUE_TYPE(pspec);
    VipsObjectClass * oclass;
    const char * name;
    name = g_type_name(otype);
    const char * owner_name;
    owner_name = g_type_name(pspec->owner_type);

    printf("%s\n", g_param_spec_get_name(pspec));
    printf("%s\n", g_param_spec_get_nick(pspec));
    printf("%s\n", g_param_spec_get_blurb(pspec));

    if (g_type_is_a(otype, VIPS_TYPE_IMAGE))
        printf("VipsImage\n");
    else if (g_type_is_a(otype, VIPS_TYPE_OBJECT) &&
        (oclass = g_type_class_ref(otype))) {
        const char * name;
        name = g_type_name(otype);
        printf("%s-%s\n", name, oclass->description);
    } else if (G_IS_PARAM_SPEC_BOOLEAN(pspec)) {
        GParamSpecBoolean * pspec_bool = G_PARAM_SPEC_BOOLEAN(pspec);

        printf("bool:%d\n",
            pspec_bool->default_value);
    } else if (G_IS_PARAM_SPEC_INT(pspec)) {
        GParamSpecInt * pspec_int = G_PARAM_SPEC_INT(pspec);

        printf("int:%d:%d:%d\n",
            pspec_int->minimum,
            pspec_int->maximum,
            pspec_int->default_value);
    } else if (G_IS_PARAM_SPEC_UINT64(pspec)) {
        GParamSpecUInt64 * pspec_uint64 = G_PARAM_SPEC_UINT64(pspec);

        printf("uint64:%"
            G_GUINT64_FORMAT ":%"
            G_GUINT64_FORMAT ":%"
            G_GUINT64_FORMAT "\n",
            pspec_uint64->minimum,
            pspec_uint64->maximum,
            pspec_uint64->default_value);
    } else if (G_IS_PARAM_SPEC_DOUBLE(pspec)) {
        GParamSpecDouble * pspec_double = G_PARAM_SPEC_DOUBLE(pspec);

        printf("double:%g:%g:%g\n",
            pspec_double->minimum,
            pspec_double->maximum,
            pspec_double->default_value);
    } else if (G_IS_PARAM_SPEC_ENUM(pspec)) {
        GParamSpecEnum * pspec_enum = G_PARAM_SPEC_ENUM(pspec);
        int i;

        printf("enum-%s\n", name);
        for (i = 0; i < pspec_enum->enum_class->n_values; i++)
            printf("%d:%s:%s\n",
                pspec_enum->enum_class->values[i].value,
                pspec_enum->enum_class->values[i].value_nick,
                pspec_enum->enum_class->values[i].value_name);
        printf("%d\n", pspec_enum->default_value);
    } else if (G_IS_PARAM_SPEC_BOXED(pspec)) {
        if (g_type_is_a(otype, VIPS_TYPE_ARRAY_INT)) {
            printf("array of int\n");
        } else if (g_type_is_a(otype, VIPS_TYPE_ARRAY_DOUBLE)) {
            printf("array of double\n");
        } else if (g_type_is_a(otype, VIPS_TYPE_ARRAY_IMAGE)) {
            printf("array of images\n");
        } else if (g_type_is_a(otype, VIPS_TYPE_BLOB) && strncmp(owner_name, "VipsProfileLoad", 100) == 0) { 
            //that's the only one that uses VipsBlob... somehow void pointers are returned as VipsBlob as well
            printf("VipsBlob\n");
        } else if (g_type_is_a(otype, VIPS_TYPE_BLOB)) {
            printf("byte-data\n");
        } else {
            printf("unsupported boxed type %s\n", name);
            vips_error_exit(NULL);
        }
    } else if (otype == 64) {
        printf("string\n");
    } else if (G_IS_PARAM_SPEC_FLAGS(pspec)) {
        GParamSpecFlags * pspec_enum = G_PARAM_SPEC_FLAGS(pspec);
        int i;
        const char * name;
        name = g_type_name(otype);
        printf("flags-%s\n", name);
        for (i = 0; i < pspec_enum->flags_class->n_values; i++)
            printf("%d:%s:%s\n",
                pspec_enum->flags_class->values[i].value,
                pspec_enum->flags_class->values[i].value_nick,
                pspec_enum->flags_class->values[i].value_name);
        printf("%d\n", pspec_enum->default_value);
    } else {
        printf("unsupported type %s \n", name);
        vips_error_exit(NULL);
    }
}

static void *
    show_required_optional(VipsObject * operation, GParamSpec * pspec,
        VipsArgumentClass * argument_class,
        VipsArgumentInstance * argument_instance, void * a, void * b) {
        gboolean required = * ((gboolean * ) a);

        if (argument_class->flags & VIPS_ARGUMENT_DEPRECATED)
            return (NULL);

        if (!(argument_class->flags & VIPS_ARGUMENT_CONSTRUCT))
            return (NULL);

        if ((argument_class->flags & VIPS_ARGUMENT_REQUIRED) == required) {
            printf("PARAM:\n");
            if (!(argument_class->flags & VIPS_ARGUMENT_INPUT))
                printf("OUTPUT:");
            show_argument(pspec, argument_class);
        }

        return (NULL);
    }

static int
usage(const char * operation_name) {
    VipsOperation * operation;
    gboolean required;

    if (!(operation = vips_operation_new(operation_name)))
        return (-1);

    printf("REQUIRED:\n");
    required = TRUE;
    vips_argument_map(VIPS_OBJECT(operation),
        show_required_optional, & required, NULL);
    printf("OPTIONAL:\n");
    required = FALSE;
    vips_argument_map(VIPS_OBJECT(operation),
        show_required_optional, & required, NULL);

    g_object_unref(operation);

    return (0);
}

static void *
    show_class(GType type) {
        if (!G_TYPE_IS_ABSTRACT(type)) {
            VipsOperation * operation;
            VipsOperationFlags flags;

            operation = VIPS_OPERATION(g_object_new(type, NULL));
            flags = vips_operation_get_flags(operation);
            g_object_unref(operation);

            if (!(flags & VIPS_OPERATION_DEPRECATED)) {
                const char * name;
                name = g_type_name(type);
                VipsObjectClass * oclass;
                oclass = g_type_class_ref(type);
                printf("OPERATION:\n%s:%s\n", oclass->nickname, name);
                vips_object_print_summary_class(
                    VIPS_OBJECT_CLASS(oclass));
                usage(name);
            }
        }

        return (NULL);
    }

int
main(int argc, char ** argv) {
    GOptionContext * context;
    GOptionGroup * main_group;

    GError * error = NULL;

    if (vips_init(argv[0]))
        vips_error_exit(NULL);

    context = g_option_context_new("vips introspection");

    main_group = g_option_group_new(NULL, NULL, NULL, NULL, NULL);
    vips_add_option_entries(main_group);
    g_option_context_set_main_group(context, main_group);

    if (!g_option_context_parse(context, & argc, & argv, & error)) {
        if (error) {
            fprintf(stderr, "%s\n", error->message);
            g_error_free(error);
        }

        vips_error_exit(NULL);
    }

    vips_type_map_all(g_type_from_name("VipsOperation"),
        (VipsTypeMapFn) show_class, NULL);

    g_option_context_free(context);

    vips_shutdown();

    return (0);
}