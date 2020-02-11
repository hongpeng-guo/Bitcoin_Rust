# Assignment 2
In this assignment, you will implement some crypto-primitives and basic data structures. You will need the code that we provide in this repo. Please follow the instructions. **Due date: 12:30PM, Feb 6, 2020.**

**Notice that this is an individual assignment. You should finish it on your own.**

## Repository management and submission:
1. Fork the current repo. **Change visibility to private, if you don't want someone see your code.** Note: We are also going to use the same repo for midterm project, which means you don't need to fork this repo again in the future.
2. Add TAs as a reporter on your forked repo (this repo is different from the one you used for assignment 1, so please add TAs again). TAs are `geruiw2` and `rbrana2`.
3. Fill in a google form to provide your repo URL. ~~The form is not yet ready and will be ready by Jan 30.~~ The link is: [https://forms.gle/BSva88p8MVCCAVPw5](https://forms.gle/BSva88p8MVCCAVPw5).
4. TAs will initiate a pull from your repo at the deadline for submission, and this will be considered a final submission.
5. Students can run tests (by command `cargo test`) provided in the code to check the validity of their implementation.
6. TAs will run additional tests (private) on the final submission to award marks.

## Code provided
We have provided incomplete code for implementing some crypto-primitives and data structures like merkle tree, signature and transactions. The following files are related to this assignment and you should read them.
1. _src/crypto/hash.rs_ - Provides __H256__ struct(32 byte array),  __Hashable__ trait, with its implementation for H256. 
2. _src/crypto/keypair.rs_ - function to randomly generate keypair.

You don't need to write anything in the above two files.

3. _src/crypto/merkle.rs_ - struct defition of **MerkleTree** struct and the related function declaration
4. _src/transaction.rs_ - struct defition of **Transaction** struct and function declaration for __sign()__ and __verify()__ .

You will write your code in the above two files.

As for other files in the repo, you don't have to worry about them in this assignment. They may appear in future assignments/projects.

## Programming
After you fork this repo, the first thing we suggest is to run command `cargo test` to see whether the code is compiling on your machine. (If compiling has error, please check the version of cargo to be the latest stable.) If the compiling is successful, you will see something like this:
```
running X tests
test XXX ... FAILED
test XXX ... FAILED
```
It's expected that tests fail with the code we provide. After you finish this assignment, some of the tests will pass.

You need to implement the missing parts in the code. They include the following.

### Transaction and signature
This part is in file _src/transaction.rs_.
1. You need to fill in the **Transaction** struct. Up to now we don’t expect the cryptocurrency and payment to be functional, so you can put any content in transactions. A simple choice is to put some **Input** and **Output** inside transactions and you can define **Input** and **Output** by yourself.
2. You need to fill in the **sign** and **verify** function. These two function should sign and verify the digital signature of the **Transaction** struct. Please use **ring** crate (actually the crate is already used in the heading of this file). The code we provide contains some `unimplemented!()` and you can delete it and write your own code.
3. A tricky part about transaction and signature is how you put them together. One possible way is to create another struct called **SignedTransaction**. The other way is to declare a field in transaction called *signature*, which will be empty if there is no signature. Feel free to design your own way. (We don’t require you to define a struct to carry signature in this assignment.)
4. For testing, you need to fill in the function **generate_random_transaction()** which will generate a random transaction on each call. It should generate two different transactions on two calls. We require this since we are going to use this function many times in our test and grading. Just a suggestion: don’t generate a very large transaction, since it will slow down our test platform. Again, there is `unimplemented!()` and you can delete it.
5. We provide a small test function named **sign_verify()**. After you finished steps 1-4, you can run `cargo test` and you can see the result of this function in the output. It will look like the following.
```
test transaction::tests::sign_verify ... ok
```
To test your code, you are free to write more tests.

### Merkle Tree
This part is in file *src/crypto/merkle.rs*. You need to complete the merkle tree struct and some functions. We covered merkle tree briefly in the lecture. You can also find a good article about it [here](https://nakamoto.com/merkle-trees/). Specifically, the functions you need to implement are:
1. *new()* - this function takes a slice of Hashable data as input, and create the merkle tree. 
2. *root()* - given a merkle tree, return the root. The computation of the root is inside *new()*, this function should just return the root.
3. *proof()* - given a merkle tree, and also given the index, this function returns the proof in the form of a vector of hashes.
4. *verify()* - given a root, a hash of datum, a proof (a vector of hashes), an index of that datum (same index in *proof()* function), and a leaf_size (the length of leaves/data in *new()* function), returns whether the proof is correct.

We provide some small test functions in this file and you can run `cargo test`. In these test functions, we also provide a brief explanation about the expected computation.

*new()* function can take any Hashable data, but for simpilicity we will test merkle tree over **H256**, whose Hashable trait is already provided inside *src/crypto/hash.rs*.

A tricky part about *new()* is when the input length is not a power of 2, you will need some more steps to create the merkle tree as follows.
> Whenever a level of the tree has odd number of nodes, duplicate the last node to make the number even.

## Advance Notice
- At the end of the midterm project, you will implement a functional cryptocurrency client. We don't require you have a functional transaction struct in this assignment, but please start to think what transaction struct should be. Also please start to think about UTXO since it is closely related to transaction.
- This code base provides other files that will help you build a blockchain client. If you want to run the main program and see what is going on, you can run `cargo run -- -vv`. Currently the main program just stucks at a loop.
