/// Formats a float as a string, ensuring that it always contains a decimal point.
/// For example, 42 will be formatted as "42.0", while 3.14 will be formatted as "3.14".
/// 
/// ```
/// assert_eq!(format_float(42), "42.0");
/// assert_eq!(format_float(3.14), "3.14");
/// ```
pub(crate) fn format_float(f: f64) -> String {
    let s = format!("{}", f);
    if s.contains('.') { s } else { s + ".0" }
}
