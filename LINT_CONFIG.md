# Configuración adicional de linting para Cargo.toml
# Añadir estas líneas al [workspace] en Cargo.toml

[workspace.lints.rust]
unsafe_code = "warn"
missing_docs = "warn"

[workspace.lints.clippy]
# Errores críticos que debemos evitar
unwrap_used = "deny"
expect_used = "warn"
panic = "deny"
todo = "warn"
unimplemented = "deny"

# Mejoras de rendimiento
clone_on_ref_ptr = "warn"
redundant_clone = "warn"
unnecessary_wraps = "warn"

# Mejores prácticas
cognitive_complexity = "warn"
too_many_arguments = "warn"
type_complexity = "warn"

# Seguridad de memoria
ptr_as_ptr = "warn"
cast_lossless = "warn"

# Claridad de código
explicit_deref_methods = "warn"
explicit_into_iter_loop = "warn"
map_unwrap_or = "warn"
