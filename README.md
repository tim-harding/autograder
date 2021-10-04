# Github Classrooms Autograder

An implementation of the Github Classrooms autograder.


## Install

You can download the latest executable for Ubuntu from the [project releases](https://github.com/tim-harding/autograder/releases/).


### Cargo

With a [Rust toolchain](https://www.rust-lang.org/tools/install) installed, run

`cargo install --git https://github.com/tim-harding/autograder`


### Script

```
git clone https://github.com/tim-harding/autograder.git
cd autograder
chmod +x ./install.sh
./install.sh
```


## Use

Navigate to the root of your assignment repository and run

`autograder`

If needed, you may specify the `autograding.json` file location with the `--config` flag. Autograder expects to be run under Ubuntu or some platform with `bash`.