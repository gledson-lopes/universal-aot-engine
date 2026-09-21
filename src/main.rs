mod arena;
use arena::Arena;

unsafe extern "C" {
    fn rt_get_tag(ptr: *const u8) -> i32;
    fn rt_get_payload(ptr: *const u8, offset: i32) -> i32;
}

pub struct Enum<'a> {
    ptr: *mut u8,
    name: &'static str,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Enum<'a> {
    fn new(arena: &'a Arena, name: &'static str, tag: u32, payload: &[i32]) -> Self {
        let ptr = arena.alloc_enum(tag, payload);
        Self {
            ptr,
            name,
            _marker: std::marker::PhantomData,
        }
    }

    fn tag(&self) -> i32 {
        unsafe { rt_get_tag(self.ptr) }
    }

    fn payload(&self, index: usize) -> i32 {
        unsafe { rt_get_payload(self.ptr, (index * 4) as i32) }
    }
}

fn main() {
    println!("\x1b[1m--- Enum (Arena) ---\x1b[0m\n");

    let arena = Arena::new(1024 * 1024);

    {
        let house = Enum::new(&arena, "Home::House", 0, &[4, 2500]);
        let apt = Enum::new(&arena, "Home::Apartment", 1, &[12, 404]);
        let card = Enum::new(&arena, "Payment::Card", 0, &[9999]);
        let color = Enum::new(&arena, "Color::Custom", 2, &[255, 128, 0]);

        println!(
            "\x1b[32m[AOT]\x1b[0m {} | Tag: {} | Rooms: {} | SqFt: {}",
            house.name,
            house.tag(),
            house.payload(0),
            house.payload(1)
        );
        println!(
            "\x1b[32m[AOT]\x1b[0m {} | Tag: {} | Floor: {} | Unit: {}",
            apt.name,
            apt.tag(),
            apt.payload(0),
            apt.payload(1)
        );
        println!(
            "\x1b[32m[AOT]\x1b[0m {} | Tag: {} | Value: {}",
            card.name,
            card.tag(),
            card.payload(0)
        );
        println!(
            "\x1b[32m[AOT]\x1b[0m {} | Tag: {} | R: {} G: {} B: {}",
            color.name,
            color.tag(),
            color.payload(0),
            color.payload(1),
            color.payload(2)
        );
    }

    println!("\n--- Scope End (arena still valid) ---");
    drop(arena);
}
