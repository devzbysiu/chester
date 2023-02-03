pub type TestRunner = Box<dyn Runner>;

pub trait Runner {
    fn run(&self);
}
