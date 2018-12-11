add task:
	parameters:
		- type
			- task or reward, maybe for prototype just have a flag that marks it as a reward but in general you could have multiple different types of tasks with different probabilities to choose. for now choosing a reward with a hardcoded percentage 
		- name
		- priority
			- lowest 1 highest infty (well, max usize i guess)
	
list tasks:
	- prints a list of tasks
		- I guess just two lists, one of tasks and one of rewards

show current task:
	- print current active task

complete task:
	- removes current task from task list
		- if task is reward keep by default?
	- maybe keep a log of completed tasks by day or something?

	parameters:
		- maybe have a --keep parameter
		- maybe use it like `tkz complete [--keep] "notes about completed task"` i.e. anything at the end is treated as a string containing notes about the task, like a git commit message

skip task:
	- returns current task to task list

	parameters:
		- maybe optional note why it was skipped
		- maybe a message if the task gets skipped a lot

---

more features:
recurring tasks like todoist
arbitrary categories
randomized timer with notification

--- 

no select task - if you finish that specific task while it isn't current, just skip it when it gets randomly selected
	- eventually i guess if there are a bunch of low priority tasks it can get cluttered so maybe delete would be useful
	- list indices on list command, use those to delete

---

I changed the terminology from reward to break because it sounds nicer but left reward as the actual variable in the code since break is a keyword

---
testing ideas:

define a trait for above database operations, then either put boxed or impl trait when used in structs/params for testing with mocks

for choosing random task, it should be easier, just put the actual randomly generated value as a parameter

---

sync ideas:

eventually if i make a web ver. and do sync, how do you do it?

"unsynced operations" table? in the case we are offline or manually sync, otherwise just do it immediately if we have connection.

fancy CDRT stuff?

add a task:
	- can always merge adds
	- if server doesn't receive acknowledgement but updates were delivered we can get duplicated adds?
		- if we check guids that isn't a problem
delete a task:
	- need to use ID, so would need to add global ids to tasks
	- if task was created and then completed while offline, or before the other party had to sync, then i guess we don't even need to send that in the first place but also it's not an error
		- if we put it in an unsynced operations table we can check if the corresponding add is also unsynced
		- probably unnecessary

---

more sync ideas:

saw in an HN comment, add an operation ID to prevent duplicate operations. if you see the operation id twice then you know the client didn't see an ack.

---

database:
ideally, all database operations should be self-contained so that you never need to handle db indices outside the actual db operation code

	!!! need to do "pragma foreign_keys = ON;" !!!

	tasks table:
		id, task, priority, category

		"create table tasks (
			id INTEGER PRIMARY KEY,
			task TEXT NOT NULL,
			priority INTEGER NOT NULL,
			category INTEGER NOT NULL
		);"

	current task table:
		foreignkey to tasks table id?
		should always have exactly 0 or 1 elements

		"create table current (
			id INTEGER PRIMARY KEY check (id = 1),
			task_id INTEGER NOT NULL,
			foreign key (task_id) references tasks(id),
		);"

	completed tasks table:
		id, task, category, date completed

		"create table completed (
			id INTEGER PRIMARY KEY,
			task TEXT NOT NULL,
			category INTEGER NOT NULL,
			date_completed TEXT NOT NULL
		);"
	
	metadata table:
		version number, creation date
		check if version number is equal to cargo version number

		"create table metadata {
			id INTEGER PRIMARY KEY check (id = 1),
			version TEXT NOT NULL,
			date_created TEXT NOT NULL
		);"

	- current task table probably saves space compared to having an extra "current" bool column and makes it easier to preserve the "only one current active task" invariant
	- completed task table also saves space and probably helps prevent bugs accidentally selecting completed tasks when selecting current from tasks table

creation operations:
- open
	probably just open or create

	for create:
		- sqlite: foreign key pragma
		- create tables

operations for each action:
- add
	- insert to database
- list
	I guess for now I can just return a tuple of vecs or something
	- get all tasks, sorted by category and then priority
	- maybe query each separately?
		- i.e.
		"select [...] from tasks where category=task order by priority"
		"select [...] from tasks where category=reward order by priority"
		
		- probably not necessary, just append to different vecs inside query_map
- show current
	- can do in one query:
	"select [...] from tasks where id =
		( select taskid from current where id = 1 )
	- if 0 results no current task set, but i don't think it's an error

- complete task
	- get current task
	- add current task to completed
	- remove current task from tasks table
		- same query as show current but delete instead of select
		- we need to return the task as well though, so get select first
	- get all tasks as in list operation
	- (not db operation) choose new task
	- set current task: "replace into current (id, task_id) values (1, :task_id);"

- skip task
	- same as complete task but without removal operation

so, operations needed:

tasks_add(Task) -> Result<(), Error>
tasks_list() -> Result<(Vec<Task>, Vec<Task>), Error>
tasks_current() -> Result<Task, Error>
tasks_remove_current() -> Result<Task, Error>
current_set(Task)
completed_add(Task) -> Result<(), Error> 
tasks_skip

---

drop impl on database struct to close?

---

I got caught up in trying to figure out the best way to do categories and not expose primary keys/ids to the user api (do you make a guid for the category, or do you just call the category string/text a "natural key" and put a UNIQUE on the name column and do queries on the text, etc) so since this is a "prototype" i'm just going to make it a bool.

I think the right way is probably to have a table that's |rowid|category|category_priority| and have a foreign key in the task table pointing to the category

then when you populate a task struct you join on the category_id and put in an actual category struct
similarly when you insert a task it only has the category name, and I guess the insert should fail if the category doesn't exist because of the foreign key check

--- 

property testing idea, I could make an arb_db() that just calls the create test db utility function, and then compose with other ops to get like "arb_db_with_tasks()" maybe

---

we can view the main function in a cli to essentially be a render function which renders by printing.

we could i guess make it more complicated and have render functions for each of the commands or something?

cli control flow is input (structopt) -> command (dispatch, business logic) -> output (display result of dispatch)

so really we could have different renders for each of the outputs of the commands

but really they're all going to output the same type (a list of strings to print out, a widget tree, whatever ncurses does, or whatever)

---

could make taskerizer per-project by putting it in a dotfile in the current directory.

interface would maybe be like `tkzr create [dir]` dir optional with default value current directory

and then all commands would first look up the tree for a .tkzr/ before looking in the default 'global' directory

alternatively use new vim model of putting per-directory files inside global dir. probably better option to do this.

add override command -g --global to tkzr, or maybe option in config

website/mobile implementation i guess would be "projects" and then the tkzr command would be `tkzr create <name> [dir]`, or maybe it just uses the name of the directory.
	this is something where i could see it as being a paid feature, much like todoist

---

Going to try to make the tests use an in-memory db because it's slow on a test machine I use. i originally didn't want to do this because then you have the same problem as using a mock: you have to test the part that you're mocking separately. so I think probably what i'll end up doing is use the in-memory db for unit tests and writing to disk for integration tests (i.e. the tests in the tests/ directory)

in order to do this I think i have to refactor the connection opening code or maybe more generally the config-based code to outside the commands. so i would acquire the db connection, and then dispatch the commands, only passing the connection and other data into the run function for each command. for the unit tests for the db I think I just need to change the call used in `src/db/tests/mod.rs:open_test_db`

I should probably add a separate test for the db that actually opens and writes to disk

---

I think the proper way to implement the "current task" is to make the db expose a "select current" method. this doesn't actually break any invariants by itself - only not using it in other places does. 

so the problem is then where should we do so? in the top-level dispatched functions or inside the db functions themselves?

that is: should the DBBackend::add_task select a current task when there are none, or should that be up to the higher level function? from a separation of concerns viewpoint it shouldn't do anything other than add a task, but from a "state-consistency" point of view it kind of should. 

given that the db module is pub(crate) and not a real "public api" i think it's okay that you can put it in a somewhat inconsistent state. the rule "when you add a new task, if there are no current tasks, select one" is kind of more a "business rule" than a technical constraint

---

the above idea has the same problem that add has: we don't actually check that add is *really* working in the add tests, we just check that it doesn't error. the tests for list task and get current task are actually testing the implementations of add as well - if add or select current task (respectively) are broken, their tests don't fail, the tests for list/get_current fail. but we can't really test list without having a working add.

i'm not really sure what the right thing to do here is. maybe there's a better, more testable design.

one thing I considered is that both add and select_current task should return the task that was added/selected, but that doesn't actually check that they were stored - only reading them back checks that they were stored, which is again testing list.

---

I suppose the right thing to do in terms of TDD would be to mock the rest of the db but 1. you would still need to define the "get/list" api in each mock and 2. you would still need to write integration tests so why write the mocks

---

as expected, when I wrote the tests and code for get_current_task, my tests are failing but I can't tell whether it's the choose_current_task code or get_current_task code.

given that the tests for choose_current_task pass with some extra checks that I added to return errors if anything other than exactly 1 row is modified, I am still pretty confident that the choose_current_task code is working but that's partially an artifact of the fact that i'm using rust rather than my testing strategy.

---

the problem was indeed in choose_current and in fact it was a feature of rust that was the problem: i made a transaction to do all the updating queries and inserts atomically, but then the transaction in rusqlite is rolled-back by default when it goes out of scope unless you explicitly call tx.commit(), consuming the transaction. i think an even more advanced type system could possibly fix this. i am not sure/ don't think it can be fixed by putting a #[must_use] annotation on the transaction struct itself. maybe it can.

---

okay second problem: if we call choose_current_task with reward: true and there are no rewards, or reward: false and there are no tasks, we return Ok. This doesn't put the database in an invalid state but it also doesn't really do what we want.

Options: return an error, or leave it and document the behavior

probably return an error: if the end result of calling "choose a new task" doesn't result in a new task being chosen, that's probably an error. 
in the higher level code we can check if there are any tasks/breaks before actually calling choose new task. e.g. grey out the box. 

in higher level code we're going to select task/break as a separate probability anyway. i.e. "if there are tasks and breaks / if there are tasks / if there are breaks / if neither" so we should always know beforehand anyway. therefore it should be an error.

---

using Task::from_parts at untrusted/io boundaries (user input, db retrieval) but now I have the problem of whether I want to test those conditions/code paths in those cases. I.e. do i want to write tests that check "if i put in the wrong input, I get the error from Task::from_parts" instead of leaving it to the fact that I'm using Task::from_parts

I'm pretty sure the answer is yes, I do want to write those tests, because 1) i already have those should_panic tests in tests/add.rs and 2) if I remove the usage of Task::from_parts or change its implementation, and I'm only testing on good input, then there's no way to notice if something goes wrong.

For the db part, I'm not really sure how good an idea it is to intentionally make a "corrupt" database for testing purposes, but in this specific case it isn't hard: I can make an invalid Task because its fields are public and add it to the database, but I also kind of want to make them non-public and only have getters and a constructor, which would mean that I would have to manually write a query to inject a bad Task, which is very fragile as a test. I guess I'll cross that bridge when I get there. 

Actually, one option would be to add an `impl Task` inside the test cfg'd task::test_utils module that has functions which make invalid Tasks. I'm going to try that for the db "corruption" tests. Yes, it worked.

---

for some reason once I changed the prop tests to use non-zero-length strings, it generated a string with a null byte for the first time and that happens to not be allowed by rusqlite (because sqlite strings are null-terminated)

maybe i should, just in general, only allow printable characters within task descriptions?

i really don't like the idea of adding redundant tests for that in three more places (again, in the Task::from_parts tests, the commands::Add tests in tests/add.rs, and then in the db::tests)

I think the best solution is probably to make the Task fields private and then only allow Tasks to be created via from_parts, which then is the only place that needs the tests because everywhere else is using already-valid Tasks.

that should be fine because in the same way you don't write tests for other libraries' code when you use them, you don't write tests for other layers' code.

to rebutt the points i made above about "if i remove the usage of task::from_parts or change its implementation, and i'm only testing on good input", the new thing that I'd be doing is that "if I have a Task, it is good input": I have/will have tests using bad input, but only for Task::from_parts, because that will be the only way to create a Task.

so now Task is the single source of truth about what is and is not a valid task. 

what i need to do is either:
 - make the tests pass (probably by changing the input as a stop-gap)
 - adding the null byte/non-printable character tests to Task
 - refactor by making the fields private, and then deleting the duplicate tests in Add and db

or go back and do the refactor first before the current changes that made the tests fail.

even though the latter would be easier i think i'll do the former because it seems like more of a "real-life" situation, where the "go back before the tests fail" is not an option.

---

I wish I could specify failing input on my tests because when I change tests, usually what I do is put an early return in the function with bad output to make it fail after the change, then removing the early return to make the test pass. this makes me feel confident I didn't accidentally mess up the test when changing it.

this is related to mutation testing.

of course, you can argue that the compiler erroring is a form of failing tests, but i would feel better if the tests themselves actually failed as well

---

now that i'm at the point where I'm going to delete the tests that check "is an error from Task::from_parts being produced when I create a task with invalid parts from the input or from the database" I'm kind of uncertain whether I want to do it. like, if your code can produce an erroryou should probably test it, but at the same time *it's already being tested*

essentially the problem is that I have

foo(input) and test_foo_bad_input()

but now I also have bar(input) {... foo(input)? ...} and I want to make sure that bar(bad_input) is returning an error. which is just duplicating the code in test_foo_bad_input()

---

how deterministic is the current function? if we do Add, Current, Add, should we always get the same result as doing Add, Add, Current?

this only happens either when we start from an empty state, or when we do Complete Add\* Current - is the task added after complete a possible result when we see current? what is natural?

when I do complete, do I expect a new one to be chosen immediately? well, what if I do a list command? it might want to indicate in some way the current value?

what about on a mobile app? if I'm on a screen that shows the current task with a big "complete" button, and i hit the button, I expect it to immediately appear. so I think a new task should be chosen immediately upon completion. 

that means that when we do the first add (or any add if we complete all tasks) we have to check if there is a current.

---

solution for "when to update current task": just do it every time. after running TKZCmd::dispatch, if no error occurred, run an "update current" function that checks if there is no current task and chooses a new one.

to test, do things as above: 

add -> check current
add+ -> check current is the first one added

add complete add -> check current
add complete -> check no error on complete, check no current
add complete add+ -> check current is the one added directly after complete

similar for skip,
add skip -> check current is same as before skip
add add skip -> TODO should skip refuse to use the same one twice? nah
skip -> check no err, check no current

---

Even less convinced of the need for the Subcommand trait in commands/mod.rs given that only some of the commands need the db to be mutable. (specifically, the ones that use choose_current_task, which needs a mutable ref because the transaction api of rusqlite needs one)

---

I realized that having the Subcommand trait is actually useful because it is not a public trait, so that even though the subcommands and all their fields have to be public (though maybe they don't), you can't accidentally call just the Subcommand::run by itself (or accidentally make the run command public), which would potentially disrupt any invariants maintained by TKZCmd::dispatch (specifically, that if there is no current task, we choose one)

---

probably need to get rid of the `make_sqlite_backend` function and just use SqliteBackend directly. maybe even get rid of dbbackend trait since i'm not using mocks or anything. maybe if sqlite didn't have an inmemory version it would have been useful.

---

regarding the tests in `commands/test_dispatch.rs`:
I think these tests probably belong here, but I'm not sure. I want to test the invariants
promised by dispatch, which so far is really just that "if there is at least one task, there
should be a current task". See L330 in notes.md.

The tests in tkzr/tests/ are more like system tests that check that the output is correct. The
tests here are more like integration tests specifically for the piece of logic that says "after
we've done an operation, make sure that a task is chosen".  Similarly, if we were to add code
that says "after we've done an operation and chosen a new task, try to sync that with a remote
server", we might try to test that here as well.

The thing is that except for the fact that we can't query the database from the system tests in
tests/, these tests could also be written just by executing Add and then Current commands and
testing that the output is correct. But we don't really want to test "the output is correct", we
want to test "the database has the correct current task". Actually, the "execute Add and then
Current commands" tests will be precisely the tests inside tests/current.rs.

---

fuzz the commands (not command line input, the stuff in commands/) to make sure operations maintain invariants? might be easier to fuzz/proptest db operations

probably should just proptest db operations more thoroughly

- list never changes the database
- adding tasks never changes current task
- completing a task only removes one task
- changing break probability... only changes the break probability

these are mostly "operation X is isolated from tables A, B, C" etc
	- it would be cool to encode that in the type system but at that point you're basically making an ORM
		- eg you would first get a "&mut Tasks" which represents the tasks table, so that you can't mutate the current table
		- or you would get a "&Current"
		- then you would perform the operations (like get_all_tasks or get_current_tasks) on those objects instead of the database in general
		- they would still probably need to be tested the same way (i.e. when you have a &Table and a &Table2 you can't change the values in &Table2 with the &Table) so i'm not sure you would gain anything
		- if you added this feature to an orm maybe you wouldn't have to write *as many* tests - you'd only have to write them once for the orm and then the type system could guarantee it the rest of the time

---

I should roll up the stuff in `TKZCmd::choose_new_current` back into `DBBackend::choose_current_task()`. in particular, I wouldn't have to implement a new db operation for counting the number of each task. 

---

I think it would be good to look into using failure more clearly. The main thing is to make it so that I don't have to `Result::map_err` everything inline. it's basically giving me a backtrace via telling me the specific line or operation in which the failure occured now, but I'm doing it manually by writing descriptions. if instead I could write the description once for each error outside of the code, and then combine them into a single error type, that would be nice, and I think that is indeed what failure's Fail trait(?) provides.

It seems specifically that the [context method](https://github.com/rust-lang-nursery/failure/blob/master/book/src/fail.md#context) is what I am talking about, but it's only slightly less verbose.

---

maybe rework "DBBackend" trait to be two parts: DBBackend and Transaction, and then implement the methods that are currently on DBBackend on Transaction.

this would also allow us to relax the "no db indicies exposed from the db" because we could make the indices opaque and only valid for the length of a transaction.

in particular, it would allow `choose_current_task` to avoid doing the calculations with `p` to determine which task to choose - we would just give it an index, and the index would always be valid because it can't outlive the transaction.  you could still have problems where you do something like `delete_id(&id)` and then `modify_id(&id)` and the id would be invalid - this could be solved by always moving the id, but then what if you have a legitimate case where you want to modify it twice?

---

So i'm replacing DBBackend with TxOperations - the set of operations you can do within a transaction scope. this way, as above, when we do "choose current task" we can instead do "get all tasks" -> use p to choose task from task list -> "set current task" with a specific taskid/rowid that's guaranteed to be valid for the transaction.

the problem is that dbbackend is used everywhere and so I'm doing a lot of refactoring, but at the same time I want to add tests that make sure the transaction is actually working transactionally. in reality it doesn't matter because i'm just wrapping the rusqlite transaction, so i wouldn't test it, but i feel like it still makes sense to have tests like

- create db
- tx start
	- add task
	- choose new current task
- drop tx without commit
- assert that there is no current task

or 

- create db
- tx start
	- add task
- commit tx
- tx start
	- add task
- drop tx without commit
- assert there is only one task

but there would be a lot of tests for that. in fact there would probably be exactly twice as many tests as i have now, given that all the tests as they currently exist simply drop the db/tx without committing.

i wonder if maybe the level of abstraction i have now is actually the correct one, and i should instead make each operation transactional, making sure to test the error cases more explicitly. 

---

I realized what I should actually do is not replace DBBackend with TxOperations, but add TxOperations and implement each backend call in terms of a transaction. then one of the invariants of the dbbackend functions would be (besides being transactional) that the current task is always set if there are any tasks left.

I think in terms of testing, I need to layer better so that I don't need to thread some arbitrary values (generated by rng) through the whole program. the code in dispatch where I generate a tuple and then pass that down through the whole program is kind of ugly. instead of testing "with this specific random value, we get this output" at every level (we should still have these tests at the lowest levels), test instead "if we add 10 tasks and then choose a new current task 100 times, we see 10 different current tasks over the course of the test. or something like when you do a minheap - add 10 tasks, complete 10 tasks, and make sure there are none left when you're done, and you hit every test.

on the other hand, I don't really want to do full on dependency injection with a registry thing just to pass in an Rng. One thought I had was to specify exactly how much randomness I need. i.e. i need "choose category with weights" and "choose task from list of tasks with weights" operations. so if i make a trait for this and then pass in something that implements that trait, in the same way that i'm passing around an `impl DBBackend`, it should work. in fact, isn't `impl Trait` kind of just the law of demeter

---

My current strategy is as follows:

- [x] Add SqliteTransaction wrapper struct
- [x] Use SqliteTransaction wrapper *inside* DBBackend, only changing the functions to take a &mut self (and the tests)
- [x] Create DBTransaction trait with somewhat simpler lower-level operations
	- DBTransaciton, like the original DBBackend operations, will take &self - this allows us to make Uids that reference its lifetime
	- `list_tasks_with_priority` or something that returns Vec<(Uid, priority)> where the Uid is bound to the lifetime of the transaction
	- Then we can use that to implement `choose_new_task`, etc. most of the other dbbackend operations are single-queries so they can mostly be the same
	- NOTE: DBTransaction should note somewhere that structs implementing it must have some defined behavior upon drop - not sure if there's a good way to encode this in the type system. by "defined behavior" i mean it should be defined the same for all structs implementing DBTransaction, and it should be rollback on drop
- [x] Add `transaction(&mut self)` to DBBackend
	- Test? maybe just a single "does this work" test, but really we're literally just calling the rusqlite Connection::transaction function so probably don't need to
- [ ] Copy code inside DBBackend into DBTransaction
	- This we can write new tests for, as above (eg begin transaction, add task, rollback. then check we have no tasks. )
- [ ] Reimplement DBBackend trait in terms of DBTransaction
	- Still need to figure out if new current should be chosen directly after every operation or not
	- e.g. add would simply be "start tx, tx.add, *choose current if none?*, commit tx" 
	- i think the best thing for "choose new current if none" is as it is currently: make a DBBackend function `update_current` which is implemented in terms of transactions. that way if (e.g. on a server) two calls come in one after another "add x add y", and the second add's transaction completes after the first add but before the first `update_current`, one of the `update_current`s will run.
	- ! *the point is that `update_current` should be idempotent* I'm just not sure that even then, arbitrary sequences of complete and add commute. eg two things in list -> "complete update add update" will not give the same results as "complete add update update"i
	- so actually i am back to making update current work as a transaction operation if I want full, well, transactionality of the dboperations.

---

testing randomizer vs doing deterministic integration tests

- turn the randomizer code into a "SelectionStrategy" interface, pass the interface into the db operations? maybe even make the db operations two-part, because they really do have two phases: do the actual operation (add/list/complete/whatever) and then update current. the second part is where we need the selectionstrategy.

so we can test the selection strategy separately, and then make a separate strategy that's like "choose the largest" or "choose the smallest"

and then when testing pass in a deterministic strategy.


