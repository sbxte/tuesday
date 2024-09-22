# Project: Tuesday

A To-Do CLI tool. Inspired by [grit](https://github.com/climech/grit)

# Building

Run cargo build to get a useable executable.

```
cargo build --release
```

# Tasks

- [ ] Use `rustyline`
- [ ] Version format updater

# Usage

To begin, add your first root node 


## Adding a root node
```
tuecli add -r "Hello world"
```


## Adding a child node

Adding a child node to a parent nodes goes like so 

```
tuecli add <message> [parent]
```
```
tuecli add "This is a child node!" 0
```

## Displaying the tree graph 

You can list out the root nodes you've made with 

```
tuecli ls
```

or you can list out nodes recursively from the root nodes 

```
tuecli ls -d 0
```

Or from a specific node 

```
tuecli ls <identifier>
```

```
tuecli ls 0
```


By default, listing from the root node uses a depth of 1, including `-d 0` (0 depth) to any `ls` query forces an infinite max depth listing


## Aliases

Tired of remembering node index numbers? You can alias them with 

```
tuecli alias <identifier> <alias> 
```

You can then access the node using its alias instead of index where ever

```
tuecli alias 0 alias 
tuecli ls alias
```

