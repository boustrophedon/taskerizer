[![Build Status](https://travis-ci.org/boustrophedon/taskerizer.svg?branch=master)](https://travis-ci.org/boustrophedon/taskerizer)

This is a prototype for Taskerizer, an app to help you pick what to do when faced with too many tasks.

# Motivation

One of my biggest personal problems is choosing something to start working on, or literally just the act of starting to do something. (See [this video for more information](https://www.youtube.com/watch?v=_Nz9-6Mp614))

I can list the things I need to do in a todo app, but then they just sit there and gather dust. I read in an article that the concept behind SuperMemo is to flip the relationship between computers and humans with regards to memory: instead of programming the computer to remember facts for you, supermemo programs you to recall the facts yourself.

Similarly, a todo app makes the computer remember the tasks you have to do, whereas Taskerizer ideally will help you choose which task to do. Of course, supermemo uses a fancy spaced-repetition algorithm to achieve this. Taskerizer just uses a randomizer with priority weights and some optional skinner box mechanics.

# Why prototype

It's not a hackathon project kind of protoype. It's more of a I don't know the best way to make it usable, or to design the code (though I think the structure I have now isn't bad), so I might entirely rewrite it later. I also kind of want to implement server sync mechanism and possibly a mobile and/or web version.

I'm also using the "prototype" name as an excuse to quell my inner desire to write every possible feature all at once.

# Usage

Taskerizer is a todo app that chooses what to do for you. You add tasks (and optionally "breaks" for taking a break), and taskerizer will choose a task randomly based on the weights given to the tasks, with a separate probability of choosing to take a break instead of a task.

See the command itself for full usage description. Briefly, you can add tasks with specified priorities, show the current task or all tasks, and mark a task as completed.

`tkzr add "some task" 10` adds a task with the description "some task" and priority 10. Mark it as a "break" (e.g. take a walk, watch a youtube video, read the news) with the "--break" or "-b" flag.
`tkzr` or `tkzr current` shows you the current task. 
`tkzr list` shows you a list of all the current tasks.
below not yet implemented:
`tkzr done` or `tkzr complete` marks the current task as complete, and chooses a new one at random.
`tkzr skip` skips the current task, returning it to the task list.
`tkzr break` skips the current task and chooses a task marked as a break at random. It is also used to change the probability with which break tasks are selected.
