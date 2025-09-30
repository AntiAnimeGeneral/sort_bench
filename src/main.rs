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

#[tokio::main]
async fn main() {
    {
        let device = WgpuDevice::default();
        let client = WgpuRuntime::client(&device);

        let len: usize = 10 << 20;

        {
            // let len: usize = 10;
            let vk = (0..len).map(|_| random::<u32>(..)).collect::<Vec<_>>();

            let mut input = client.create(u32::as_bytes(&vk));

            println!("preheat");
            client.sync().await;
            measure_time!({
                sort::radix_sort::<WgpuRuntime, u32>(&client, len, &mut input, None).await;
                client.sync().await;
            });
        }

        println!("bench");
        let mut sum = Duration::default();
        for _ in 0..50 {
            let vk = (0..len).map(|_| random::<u32>(..)).collect::<Vec<_>>();

            let _buffer = client.empty(len * mem::size_of::<u32>());
            let mut data = client.create(u32::as_bytes(&vk));

            client.sync().await;
            sum += measure_time!({
                sort::radix_sort::<WgpuRuntime, u32>(&client, len, &mut data, None).await;
            })
            .1;

            let trueoutput = client.read_one(data);
            let input_ = u32::from_bytes(&trueoutput);
            if !input_.is_sorted() {
                panic!("not sorted");
            }
        }
        println!("avg: {:?}", sum / 50);
    }
}
