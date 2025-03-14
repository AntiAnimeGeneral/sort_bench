#![feature(random)]

use cubecl::wgpu::WgpuRuntime;
mod sort;

#[macro_export]
macro_rules! measure_time {
    ($block:block) => {{
        use std::time::Instant;
        let start = Instant::now();
        let result = $block;
        let duration = start.elapsed();
        println!("time: {:?}", duration);
        (result, duration)
    }};
}

use std::{mem, random::random, time::Duration, u32};

use cubecl::{prelude::*, wgpu::WgpuDevice};

fn main() {
    {
        let device = WgpuDevice::default();
        let client = WgpuRuntime::client(&device);

        let len: usize = 1 << 20;

        {
            // let len: usize = 10;
            let vk =
                (0..len).map(|_| random::<u32>()).collect::<Vec<_>>();

            let mut input = client.create(u32::as_bytes(&vk));

            println!("preheat");
            measure_time!({
                sort::radix_sort::<WgpuRuntime, u32>(
                    &client, len, &mut input, None,
                )
            });
        }

        println!("bench");
        let sum = (0..100)
            .map(|_| {
                let vk = (0..len)
                    .map(|_| random::<u32>())
                    .collect::<Vec<_>>();

                let mut input = client.create(u32::as_bytes(&vk));
                let output =
                    client.empty(len * mem::size_of::<u32>());

                let time = measure_time!({
                    sort::radix_sort::<WgpuRuntime, u32>(
                        &client,
                        len,
                        &mut input,
                        Some(output),
                    )
                })
                .1;
                let trueoutput = &client.read_one(input.binding());
                let input_ = u32::from_bytes(&trueoutput);
                if !input_.is_sorted() {
                    panic!("not sorted");
                }
                time
            })
            .sum::<Duration>();
        println!("avg: {:?}", sum / 100);
    }
}
