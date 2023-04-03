// A kit for one unittest case
pub struct TestKit<T> {
    // Arg prepare for test input
    pub arg: Option<T>,
    // Arg Builder which will generate input data for test
    pub arg_builder: Option<Box<dyn Fn() -> T>>,
    // Assertion function
    pub assertions: Box<dyn Fn(&Self)>,
}

impl<T> TestKit<T> {
    pub fn run(&self) {}

    pub async fn run_async(&self) {}
}
