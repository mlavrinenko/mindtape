//! The one integration-test binary of `mindtape`: each file beside this one is a
//! module of it, so cargo links the test dependencies once, not once per file.

mod dogfood_check;
mod imperative_title;
mod schema_descriptions;
