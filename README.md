This is a WIP crate to access the Qualcomm SNPE SDK from Rust. It wraps the headers using bindgen, then exposes a Rust API on top.

Target SDK version: 2.26.0

### Building

1. Download the SNPE SDK from [here](https://www.qualcomm.com/developer/software/neural-processing-sdk-for-ai).
2. Unzip it to a directory `/path/to/sdk`.
3. Set the environment variable to point to the root directory (should have License.pdf in it).
    ```bash
    $ export SNPE_ROOT=/path/to/sdk/root
    ```
    Or alternatively, you can use the Docker container with a volume mount
    ```bash
    $ docker built -t snpe-rust .
    $ docker run -it -v /path/to/sdk/root:/snpe snpe-rust
    ```

### Plan

My plan is to support the asynchronous PSNPE runtime using `tokio` and futures, and allow running models on tensors from `tensor-rs`.
