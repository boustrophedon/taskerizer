[![Build Status](https://travis-ci.org/boustrophedon/taskerizer-prototype.svg?branch=master)](https://travis-ci.org/boustrophedon/taskerizer-prototype)

This is a prototype for Taskerizer, an app to help you pick what to do when faced with too many tasks.

# Motivation

One of my biggest personal problems is choosing something to start working on, or literally just the act of starting to do something. (See [this video for more information](https://www.youtube.com/watch?v=_Nz9-6Mp614))

I can list the things I need to do in a todo app, but then I just let them sit there and gather. I read in an article that the concept behind SuperMemo is to flip the relationship between computers and humans with regards to memory: instead of programming the computer to remember facts for you, supermemo programs you to recall the facts yourself.

Similarly, a todo app makes the computer remember the tasks you have to do, whereas Taskerizer ideally will help you choose which task to do. Of course, supermemo uses a fancy spaced-repetition algorithm to achieve this. Taskerizer just uses a randomizer with priority weights and some optional skinner box mechanics.

# Why prototype

It's not a hackathon project kind of protoype. It's more of a I don't know the best way to make it usable, or to design the code (though I think the structure I have now isn't bad), so I might entirely rewrite it later. I also kind of want to implement sync and possibly a mobile and/or web version.

I'm also using it as an excuse to quell my inner desire to architect everything out with all possible features at once, which is honestly how I should always write code.

# Usage

See the command itself for full usage description.

Taskerizer is a todo app that chooses what to do for you. You add tasks (and optionally "breaks" for taking a break), and taskerizer will choose a task randomly based on the weights given to the tasks, with a separate probability of choosing to take a break instead of a task.

In short, there are a few operations:

`tkzr current` or `tkzr` by itself shows you the current task. 
`tkzr add` lets you add a task. You can add a task categorized as a "break" using the --break flag.
`tkzr list` shows you a list of all the current tasks.
`tkzr complete` marks the current task as complete, and chooses a new one at random.
`tkzr skip` skips the current task, returning it to the task list.
`tkzr break` skips the current task and chooses a task marked as a break at random. It is also used to change the probability with which break tasks are selected.
