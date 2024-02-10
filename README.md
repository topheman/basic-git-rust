# basic-git-rust

This is a very basic and partial rust implementation of git.

The goal of this project is to get better at rust programming, not to make a full featured git implementation.

I've already done a few rust projects (with WebAssembly or bevy), however, I wanted something that needs to deal directly with:

- I/O
- parsing
- handling bytes
- parallelism

This is a work in progress, we'll see where it goes.

## Setup

Add the following alias to your `.zshrc` / `.bashrc`, it will:

- lets you use the `basic-rust-git` implementation from anywhere by using the alias `mygit`
- it runs

```sh
alias mygit=/path/to/your/repo/mygit.sh
```

## Utilities

If you cloned the repository from github, the git objects (commits, tree, ...) will be packed into `.git/objects/pack` and not present in `.git/objects` as individual objects.

Run the following to unpack the objects:

```sh
./unpack.sh
```

## Features

The features won't be implemented in full.

For the moment, the commands don't handle packed objects, so you may need to run `./unpack.sh` on the repo if you just cloned it if you want to run commands against it.

### mygit cat-file

[git cat-file - doc reference](https://git-scm.com/docs/git-cat-file)

```sh
mygit cat-file -p 342293066bfc04c621eb7cbe4d5fc3a272ff0b05
```

## Genesis of the project

It started with the ["Build Your Own Git" Challenge](https://codecrafters.io/challenges/git) from code codecrafters.io, after the first two steps, it requires a 40$/months membership - it's a little too much for the usage I would have had.
