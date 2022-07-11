# liblmod - Library for loading Linux kernel modules

### Features:
- modprobe
- rmmod

### Example code:
```rust
extern crate liblmod;

fn main() -> std::io::Result<()> {
    println!("Unloading module kvm");
    liblmod::rmmod("kvm".to_string(), liblmod::Flags::Force)?;

    println!("Loading module kvm");
    liblmod::modprobe("kvm".to_string(), "".to_string(), liblmod::Selection::Current)
}
```