# Github Classrooms Autograder

An implementation of the Github Classrooms autograder.


## Install

You can download the latest executable for Ubuntu from the [project releases](https://github.com/tim-harding/autograder/releases/).


### Cargo

With a [Rust toolchain](https://www.rust-lang.org/tools/install) installed, run

`cargo install --git https://github.com/tim-harding/autograder`


### Script

```
curl https://raw.githubusercontent.com/tim-harding/autograder/master/install.sh
chmod +x ./install.sh
./install.sh
```


## Use

Navigate to the root of your assignment repository and run

`autograder`

For more information about command line options, run

`autograder --help`