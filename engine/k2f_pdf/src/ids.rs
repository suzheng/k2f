use pdf_writer::Ref;

pub struct Alloc {
    next: i32,
}

impl Alloc {
    pub fn new() -> Self {
        Self { next: 1 }
    }

    pub fn bump(&mut self) -> Ref {
        let id = Ref::new(self.next);
        self.next += 1;
        id
    }
}
