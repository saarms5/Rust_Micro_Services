# HAL Crate: Hardware Abstraction Layer

## Overview

The `hal` crate provides safe, Rust-idiomatic abstractions for microcontroller hardware, ensuring type safety and preventing common embedded development errors.

## Architecture

### Safe Register Access (`registers` module)

The `Register<T>` generic wrapper provides type-safe access to memory-mapped MCU registers:

```rust
use hal::Register;

// Create a register from a memory address (unsafe operation contained)
let mut reg = unsafe { Register::new(0x4000_0000 as *mut u32) };

// Read register value (volatile read)
let value = reg.read();

// Write register value (volatile write)
reg.write(0xDEADBEEF);

// Read-modify-write pattern
reg.modify(|val| val | 0x01);
```

**Safety guarantees:**
- All volatile memory access is contained within the `Register` wrapper
- No unsafe blocks exist in application code
- Compiler cannot optimize away critical register operations
- `PhantomData` ensures type safety across different register types

### Peripheral Abstractions (`peripherals` module)

#### GPIO Pin Control

```rust
use hal::GpioPin;

let mut pin = GpioPin::new(5); // GPIO pin 5
pin.set_high()?;   // Drive pin HIGH
pin.set_low()?;    // Drive pin LOW
pin.toggle()?;     // Toggle pin state
```

#### UART Serial Communication

```rust
use hal::UartPort;

let mut uart = UartPort::new(0, 115200);
uart.open()?;
uart.write(b"Hello, MCU!")?;
let data = uart.read()?;
uart.close()?;
```

#### SPI Interface

```rust
use hal::SpiInterface;

let mut spi = SpiInterface::new(0, 1_000_000);
spi.initialize()?;
let response = spi.transfer(&[0xAA, 0xBB])?;
spi.deinitialize()?;
```

#### Timer/Counter Units

```rust
use hal::TimerUnit;

let mut timer = TimerUnit::new(0, 16); // Timer 0, prescaler 16
timer.start()?;
timer.set_interval_ms(1000)?;
// ... use timer
timer.stop()?;
```

## Design Principles

1. **Type Safety**: All register access is wrapped in generic types; no raw pointers leak to application code.
2. **Volatile Access**: Uses `std::ptr::read_volatile` and `std::ptr::write_volatile` to prevent compiler optimizations.
3. **Zero-Cost Abstractions**: Register wrappers compile to direct assembly instructions (no runtime overhead).
4. **Error Handling**: All I/O operations return `Result<T, String>` for graceful error handling.
5. **No Unsafe in User Code**: Application developers never need to write unsafe blocks.

## Integration with embedded-hal

The `hal` crate uses `embedded-hal` 1.0 traits to ensure compatibility with ecosystem crates (RTIC, probe-run, etc.). Peripherals can be adapted to `embedded-hal` traits where needed.

## MCU-Specific Implementation

To adapt this HAL for a specific MCU (e.g., STM32, Nordic nRF, RISC-V):

1. **Use `svd2rust`** to generate register definitions from the MCU's SVD file.
2. **Wrap SVD registers** in the `Register<T>` type.
3. **Implement peripherals** (GPIO, UART, SPI, Timer) using the MCU's register layout.
4. **Maintain the `embedded-hal` trait interface** for compatibility.

Example (pseudo-code for STM32):
```rust
// From svd2rust
use stm32f4::stm32f429::{GPIOA, RCC};

// Wrap in our safe abstraction
let mut pa = unsafe { Register::new(&stm32f429::GPIOA as *const _ as *mut u32) };
// Now use pa through the Register interface
```

## Safety and Testing

- All register operations are wrapped and contain volatile access internally.
- Peripheral operations check state (e.g., UART must be open before write).
- Tests verify bit manipulation and register field extraction patterns.

Run tests with:
```bash
cargo test -p hal
```

