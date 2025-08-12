# nlabapi
[![Crates.io](https://img.shields.io/crates/v/nlabapi)](https://crates.io/crates/nlabapi)
![PyPI - Version](https://img.shields.io/pypi/v/nlabapi)
[![License](https://img.shields.io/crates/l/nlabapi)](LICENSE)

[![Documentation](https://docs.rs/nlabapi/badge.svg)](https://docs.rs/nlabapi)
![Cargo Test](https://github.com/nLabs-nScope/nlabapi/actions/workflows/tests.yml/badge.svg)
![Python CI](https://github.com/nLabs-nScope/nlabapi/actions/workflows/build_python.yml/badge.svg)

Low-level software interface for nLab devices. Libraries for Python and Rust are available. The recommended easy way to access data straight from the nLab is the python interface, provided by the `nlabapi` python package.

## Python Usage

1. Install the `nlabapi` python package

On an existing python installation, use `pip` to install the nlabapi package
```shell
$ pip install nlabapi
```

2. Test Import and Usage of `nlabapi`
Entering the above one-liner python program using `python -c` should print a list of all connected nLabs.

```shell
$ python -c "import nlabapi; nlabapi.LabBench.list_all_nlabs()"
Link to nLab v2 [ available: true ]
```

3. Write your own scripts, or use the examples
```shell
$ python examples/list_all_nlabs.py
List of all detected nScopes:
Link to nLab v2 [ available: true ]
```

## Building from Source

The nLab API can be built and run from source to enable users and developers to quickly iterate on nLab source code. To establish a development environment, follow the steps below.


### Prerequisites

1. Rust Toolchain (https://rustup.rs)

After installing the development dependencies, check to make sure you have a working environment by running version commands for each of the required tools.

```shell
$ rustup --version
```
The above commands should print a version successfully.

> **Note** - macOS specifics
>
> On macOS the project is configured to build a universal binary, including both x86 and Apple Silicon binaries in one. To enable that, we must add both rust target toolchains as follows:
> ```shell
> rustup target add x86_64-apple-darwin
> rustup target add aarch64-apple-darwin
> ```

> **Note** - Linux specifics
>
> On linux distributions, we need the system library headers for `libusb` and `libudev`. To install these on an Ubuntu distribtion, the following command should work.
> ```shell
> sudo apt-get install libusb-1.0-0-dev libudev-dev
> ```
> On other distributions, developers should look to their package managers for these development headers.

### Clone and Install Development Dependencies

```shell
$ git clone https://github.com/nLabs-nScope/nlabapi.git
$ cd nlabapi
```

### Build and Run
```shell
$ cargo build
$ cargo run --example list_all_nscopes
```

## Python Development

This project also supports a python interface to the nLab. To set up an environment for python development, follow the steps below.

### Prerequisites

1. All steps above
2. A Python 3.9 or newer installation

### Create a virtual environment and activate it
```shell
$ python3 -m venv venv
$ source venv/bin/activate
```

### Install python dependencies
```shell
$ pip install maturin
```

### Build and install the nlabapi package for developement
```shell
$ maturin develop
$ python examples/list_all_nlabs.py 
```
