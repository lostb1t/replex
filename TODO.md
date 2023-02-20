create an Request proxy wrapper. Then we can add stuff like client_ip, host ip etc to it


profiling: https://crates.io/crates/criterion
cause its sloowwww.

better error handling/ Dont use unwra everywhere. Bubble it up as an actual http response: https://docs.rs/axum/latest/axum/error_handling/ 