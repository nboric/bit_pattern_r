use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};

const N_BYTES: usize = 10000000;
const BATCH_SIZE: usize = 1000;

struct Method {
    name: &'static str,
    total_counter: u32,
    total_time: Duration,
    method: Box<dyn MethodImplementation>,
}

trait MethodImplementation {
    fn pattern_match(&mut self, sample: u8) -> u32;
}

struct Method1 {
    pos: i8,
}

impl MethodImplementation for Method1 {
    fn pattern_match(&mut self, sample: u8) -> u32 {
        let mut counter = 0;
        for i in (0..8).rev() {
            let bit = sample & (1 << i);
            match self.pos {
                0 | 1 => {
                    if bit != 0 {
                        self.pos += 1;
                    } else {
                        self.pos = 0;
                    }
                }
                2 => {
                    if bit != 0 {
                        // do nothing
                    } else {
                        counter += 1;
                        self.pos = 0;
                    }
                }
                _ => {
                    // bug
                }
            }
        }
        counter
    }
}

struct Method2 {
    prev: u8,
}

impl MethodImplementation for Method2 {
    fn pattern_match(&mut self, sample: u8) -> u32 {
        let mut counter = 0;
        let combined_samples = ((self.prev as u16) << 8) | (sample as u16);
        for i in (0..8).rev() {
            if (combined_samples >> i) & 0x007 == 0b110 {
                counter += 1;
            }
        }
        self.prev = sample;
        counter
    }
}

struct Method3 {
    prev: u8,
    count_lut: [u32; 1024]
}

impl MethodImplementation for Method3 {
    fn pattern_match(&mut self, sample: u8) -> u32 {
        let combined_samples = (((self.prev as u16) << 8) | (sample as u16)) & 0x3FF;
        let counter = self.count_lut[combined_samples as usize];
        self.prev = sample;
        counter
    }
}

impl Method3 {
    fn new() -> Self {
        let mut count_lut = [0; 1024];
        let mut combined_samples: u16 = 0;
        while combined_samples < count_lut.len() as u16 {
            let mut method2 = Method2 { prev: 0 };
            method2.pattern_match((combined_samples >> 8) as u8);
            count_lut[combined_samples as usize] = method2.pattern_match((combined_samples & 0xFF) as u8);
            combined_samples += 1;
        }
        Self {
            prev: 0,
            count_lut
        }
    }
}

fn main() -> Result<(), std::io::Error> {
    let mut methods: Vec<Method> = Vec::new();
    methods.push(Method {
        name: "StateMachine",
        total_counter: 0,
        total_time: Default::default(),
        method: Box::new(Method1 { pos: 0 }),
    });
    methods.push(Method {
        name: "SlidingBitmask",
        total_counter: 0,
        total_time: Default::default(),
        method: Box::new(Method2 { prev: 0 }),
    });
    methods.push(Method {
        name: "LUT",
        total_counter: 0,
        total_time: Default::default(),
        method: Box::new(Method3::new()),
    });

    for _ in 0..N_BYTES / BATCH_SIZE {
        let mut rng = File::open("/dev/urandom")?;
        let mut buffer = [0u8; BATCH_SIZE];
        rng.read_exact(&mut buffer)?;

        for method in methods.iter_mut() {
            let start = Instant::now();
            for i in 0..BATCH_SIZE {
                method.total_counter += method.method.pattern_match(buffer[i]);
            }
            let end = Instant::now();
            let elapsed = end.duration_since(start);
            method.total_time += elapsed;
        }
    }

    for method in methods.iter() {
        println!(
            "Method {} total count: {}, time: {:?}",
            method.name, method.total_counter, method.total_time
        );
    }

    Ok(())
}