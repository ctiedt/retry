use retry::retry;

static mut X: u32 = 0;

#[retry(2)]
fn might_fail() -> Result<u32, &'static str> {
    match unsafe { X } {
        0..=1 => {
            unsafe { X += 1 };
            Err("Error")
        }
        _ => Ok(unsafe { X }),
    }
}

fn main() {
    let f = might_fail().unwrap();
    dbg!(f);
}
