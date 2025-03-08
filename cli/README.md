# Building and Installation 

```
cargo build -r
```
or alternatively, you could cargo install it.

# Usage

Run `cargo install` on the `cli` directory to install `tuecli`.
```
cargo install --path cli 
cd ~
```

## Adding a root node

To begin, add your first root node:
```
tuecli add -r "To-do's"
```


## Adding a child node

Adding a child node to a parent nodes goes like so:

```
tuecli add <message> [parent]
```
```
tuecli add "This is a child node!" 0
```

## Displaying the tree graph 

You can list out the root nodes you've made with:

```
tuecli ls
```

or you can list out nodes recursively from the root nodes:

```
tuecli ls -r
```

Or from a specific node:

```
tuecli ls <identifier>
```

```
tuecli ls 0
```


By default, listing from the root node uses a depth of 1, including `-r` (enable recursion) to any `ls` query forces an infinite max depth listing.

## Removing Nodes

To remove a node, enter:
```
tuecli rm <identifier...>
```

```
tuecli rm 0

tuecli rm alias

tuecli rm 1 2 3
```


## Aliases

Tired of remembering node index numbers? You can alias them with:

```
tuecli alias <identifier> <alias> 
```

You can then access the node using its alias instead of index whereever:

```
tuecli alias 0 alias 
tuecli ls alias
```

## Date Nodes

Date nodes are meant to be used as day-to-day planner.

To begin, add a node for whenever you're reading this:

```
tuecli add -d 2025-01-01
```

Alternatively, relative and human-readable dates are also supported:
```
tuecli add -d today

tuecli add -d tomorrow

tuecli add -d "2 days"

tuecli add -d "next week"
```

Now, add a few tasks for today:
```
tuecli add "task one" today
tuecli add "task two" today
tuecli add "task three" today
```

and you can list its children by either writing out its index or writing the date:
```
tuecli ls "today"
```

Note that if you have a node (normal or date node) aliased as "today", it will be prioritized first. To override this behavior, specify the -D flag.

You can also label your date nodes if you want:
```
tuecli rename today "My label"
```

Or add the label when you first add the node:
```
tuecli add -d today "My label"
```

## Linking Nodes

Tuesday stores its nodes in a multigraph data structure. You can have more than one parents or children for each node.

An example realistic use case for this is to have the complete list of your tasks under a root node that you can link to a date node for day-to-day planning.

For example, let's say you have a research project. We'll add the to-do's under a root node called "College":
```
$> tuecli add -r college
(2) -> (root)

$> tuecli add -d today
(3) -> (dates)

$> tuecli add "big research project" 2
(4) -> (2)

$> tuecli add "gather sample data" 4
(5) -> (4)

$> tuecli add "write report" 4
(6) -> (4)
```


## Calendar
Calendar with completion statistics is available as a complement for the date nodes feature. To open it, simply type:

```
tuecli cal
```

Which will bring up the calendar for the month you're currently in.

You can also specify any month of the year:

```
tuecli cal February
```

Or a specific date. Note that only its month will be considered:
```
tuecli cal 2025-02-01
```

Due to limitations, date expressions like "Feb 2025" are not supported yet. We plan to add this in the future, or you can also contribute to the codebase :)

# More Usage

Refer to the help message when you type `tuecli --help` for full usage guide.
