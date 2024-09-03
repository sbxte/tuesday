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
tuesday add -r "Hello world"
```


## Adding a child node

Adding a child node to a parent nodes goes like so 

```
tuesday add -p <parent identifier> <message>
```
```
tuesday add -p 0 "This is a child node!"
```

## Displaying the tree graph 

You can list out the root nodes you've made with 

```
tuesday ls
```

or you can list out nodes recursively from the root nodes 

```
tuesday ls -d 0
```

Or from a specific node 

```
tuesday ls <identifier>
```

```
tuesday ls 0
```


By default, listing from the root node uses a depth of 1, including `-d 0` (0 depth) to any `ls` query forces an infinite max depth listing


## Aliases

Tired of remembering node index numbers? You can alias them with 

```
tuesday alias <identifier> <alias> 
```

You can then access the node using its alias instead of index where ever

```
tuesday alias 0 alias 
tuesday ls alias
```

