[![Build Status](https://travis-ci.org/boustrophedon/taskerizer.svg?branch=master)](https://travis-ci.org/boustrophedon/taskerizer)

This is a prototype for Taskerizer, an app to help you pick what to do when faced with too many tasks.

# Motivation

One of my biggest personal problems is choosing something to start working on, or literally just the act of starting to do something. (See [this video for more information](https://www.youtube.com/watch?v=_Nz9-6Mp614))

I can list the things I need to do in a todo app, but then they just sit there and gather dust. I read in an article that the concept behind SuperMemo (a spaced-repetition flashcard program) is to flip the relationship between computers and humans with regards to memory: instead of programming the computer to remember facts for you, supermemo programs you to recall the facts yourself.

Similarly, a todo app makes the computer remember the tasks you have to do, whereas Taskerizer ideally will help you choose which task to do. Of course, supermemo uses a fancy spaced-repetition algorithm to achieve this. Taskerizer just uses a randomizer with priority weights and some optional skinner box mechanics.

This is something, in retrospect, I wish I had in college.

# Usage

Taskerizer is a todo app that chooses what to do for you. You add tasks (and optionally "breaks" for taking a break), and taskerizer will choose a task randomly based on the weights given to the tasks, with a separate probability of choosing to take a break instead of a task.

See the command itself for full usage description. Briefly, you can add tasks with specified priorities, show the current task or all tasks, and mark a task as completed.

`tkzr add "some task" 10` adds a task with the description "some task" and priority 10. Mark it as a "break" (e.g. take a walk, watch a youtube video, read the news) with the "--break" or "-b" flag.
`tkzr` or `tkzr current` shows you the current task. 
`tkzr list` shows you a list of all the current tasks.
`tkzr complete` marks the current task as complete, and chooses a new one at random.
`tkzr skip` skips the current task, returning it to the task list.
below not yet implemented:
`tkzr break` skips the current task and chooses a task marked as a break at random.

# Network sync

This is the main feature currently in progress: network sync via CRDTs.

## What is a CRDT?
[Conflict-free replicated data types](https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type) are distributed data structures that allow for eventual consistency with concurrent updates and arbitrary network partitions.

There are two major forms that a CRDT can take (although things like delta-crdts combine the two): state-based or operation-based. The difference is pretty straightforward. A state-based CRDT achieves consistency by transferring the state of the data structure and using a merge function that satisfies some properties, whereas an operation-based CRDT achieves consistency by communicating the operations performed on the data structure, and making sure the operations satisfy some properties. I think but I'm not sure that technically operation-based CRDTs are more powerful (e.g. an operation-based G-counter is easier to implement than a state-based one), but they also require more complexity in implementation. For example, usually in order not to send duplicate messages, you need to know who you're sending messages to and keep track of who has recieved them. With state-based CRDTs, you can blast state to anyone any number of times and it will work out.

The most basic example of CRDT is a "stuck light switch": You have a bool, and if you turn it on it stays on - there is no "turn off" operation. This will never lead to an inconsistent state: if you haven't observed an "on" state, it will not be on, and if you have, you will not observe it being turned off. In terms of a state-based CRDT, the merge operation is logical or of the two states. An op-based CRDT simply has the message "turn on".

## CRDTs in Taskerizer

The current design for sync is very simple: We only sync the actual task list and ignore the current task. It's fine to ignore the current task because you may want to have different current tasks in different contexts. You can compare this to Google Docs: users expect all their documents are synced, but users don't expect the last document they were working on to be already open when they open Google Docs.

The task list is represented by a set of UUIDs because we allow for duplicate tasks in our task list. Because we do not expect users to have significantly large amounts of tasks (e.g. 10k UUIDs is only 160kb), one option is to use a [two-phase set](https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type#2P-Set_(Two-Phase_Set)). We keep separate grow-only sets, one for "added" tasks and one for "removed" tasks. Then we can always compute the current active tasks by `added - removed`, where `-` here represents set difference. The set difference operation could be easily done in SQL relatively quickly, or they could be computed once at sync time. 2P-Sets have a restriction: you can't add elements you've already removed. This is not a problem in our case because elements are distinguished by UUIDs, i.e. every task is unique even if the description and priority and such are the same.

However, because we use UUIDs, we can do something simpler: We can make a unique-set CRDT, or U-set. This is described on page 23, specification 13 of the paper ["A comprehensive study of Convergent and Commutative Replicated Data Types"](https://hal.inria.fr/inria-00555588/document) by Marc Shapiro, Nuno Preguiça, Carlos Baquero, Marek Zawirski.

The general idea here is that, as long as we know the clients that need to recieve the messages, all we need to do are send "add" and "remove" messages. This seems a bit counterintuitive, because normally the add and remove methods of a set don't commute: `add(x) -> remove(x)` leaves the set without `x`, whereas `remove(x) -> add(x)` either leaves the set in an error state (if x is not in the set) or the set contains `x`.

However, since each element is unique, we can never have remove(x) first: nobody but the producer can name x. The restriction here is that the operations have to be delivered in causal order: if you do an add and then a remove locally, and then delivered the operations, you must send the add operation first. Additionally, since each x is unique, a removed element can never be added again, as in the 2P set. Therefore, duplicate removes have no effect: if it's not in the set currently, it's already been added and removed once. So we avoid the error state of "remove x when it's not in the set" by redefining it to not be an error.

For simplicity my network topology will be just a client-server/hub-spoke setup, but in general this (and any operation-based CRDT) works as long as you know all the replication members. In this case, each client thinks the only other member is the server, but the server knows all of the clients, and must retain all operations until every operation is delivered to every client.

## Network sync api specification

### Network API

- register:
	parameters: none
	response: user UUID, user key
- sync:
	parameters: user UUID, user key, list of locally-generated add/remove operations
	response: list of undelivered add/remove operations from other clients

### Operation specification

It's literally just:

```rust
enum USetOpData {
	Add (Task),
	Remove (Uuid),
}

struct USetOpMsg {
	op: USetOp,
	deliver_to: ClientUuid,
}
```


## Example:

	client A registers
	client A adds task with uuid=1
	client A adds task with uuid=2
	client A syncs
Now the server state contains tasks 1 and 2.
	client B registers
	client B syncs
Now everything is fully replicated, client A, the server, and client B all have the same state.
	client B removes task with uuid=1
Uh oh! Client A is out of date!
	client A removes task with uuid=1
Oh no!
	client A syncs
Now the server state contains only task 2
	client B syncs
The server delivers the remove op to client B during this sync, but nothing happens since uuid=1 is not there.
	client A adds task with uuid=3
	client A syncs
The server delivers the remove op from B, nothing happens because A doesn't have it anymore

# Other features I'd maybe like to do

- Compact the add/remove operations. Remove duplicate 'remove' operations i.e. (remove uuid=X, deliver to client Y), (remove uuid=X, deliver to client Y) in server's queue, remove undelivered add/remove pairs (add uuid=X, deliver to client Y), (remove uuid=X, deliver to client Y). These optimizations should preserve the CRDT.
- Explicitly time-box all tasks and automatically skip to the next task when time is up.
	- I suppose we would simply ask "did you complete this task? y/N" upon the next execution of any tkzr command, and then choose the next task.
- Projects, either per directory with a separate db file and we search up the directories for a .tkzr directory, or add more categories than "task/break" and a command to set the current category
