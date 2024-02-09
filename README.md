# basic-rust-git

This is a very basic and partial rust implementation of git.

The goal of the project is to get better at rust programming.

It started with the ["Build Your Own Git" Challenge](https://codecrafters.io/challenges/git) from code codecrafters.io, after the first two steps, it requires a 40$/months membership - it's a little too much for the usage I would have had.

This is a work in progress, we'll see where it goes.

## Setup

Add the following alias to your `.zshrc` / `.bashrc`, it will:

- let you use the `basic-rust-git` implementation from anywhere by using the alias `mygit`
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
