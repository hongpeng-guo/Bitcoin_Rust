# Midterm Project, Part 1

For midterm project, you are going to build a simplified Bitcoin client. The goal of the client is not to run in Bitcoin mainnet or any public testnet. Instead, the goal is to run it inside your team and let you have fun with it. You have plenty of freedom of designing and implementing this project.

The midterm project should be based on your code of assignment 2. You are free to discuss assignment 2 or merge codes of teammates after its due (12:30PM, Feb 6). Also please keep the code for assignment 2 and not push code for midterm project until its due. (You can use another branch rather than *master*.)

This is the first part of midterm project. You are going to finish the **Block** struct and the **Blockchain** struct. Please work in teams of 2.

**Updated: Due date: 12:30PM, Feb ~~11~~ 13, 2020.** This part of project is not as heavy as previous assignment, and we believe 5 days are enough.

## Repository management and submission

1. We suggest you to continue to work on your repo of assignment 2. Team members should work on one same repo.
2. Remember to change visibility of your repo to private.
3. Add TAs as a reporter if you haven't done it. TAs are `geruiw2` and `rbrana2`.
4. Fill in a google form to provide your repo URL. One team just need to fill it once. This is the link: [https://forms.gle/RTBHo8tzAP5LKPfy9](https://forms.gle/RTBHo8tzAP5LKPfy9)
5. Students can run tests (by command cargo test) provided in the code to check the validity of their implementation.
6. TAs will run additional tests (private) on the final submission to award marks.

## Code provided
The code we provide for midterm project is the same as assignment 2. The following files are related to this assignment.
1. *src/block.rs* - please finish **Block** struct and some functions related to it in this file.
2. *src/blockchain.rs* - please finish **Blockchain** struct and some functions related to it in this file.

## Programming

### Block

You need to define a **Block** similar to that in Bitcoin. We require that a block must include:
1. parent - a hash pointer to parent block. Please use **H256** that we provide.
2. nonce - a random integer that will be used in proof-of-work mining. We suggest to simply use **u32**.
3. difficulty - the mining difficulty, i.e., the threshold in proof-of-work check. Please use **H256** since we provide the comparison function, with which you can simply write `if hash <= difficulty`. (Proof-of-work check or mining is not required in this part.)
4. timestamp - the timestamp when this block is generated. This is used to decide the delay of a block in future part of this project.
5. merkle\_root - the Merkle root of data (explained below in 6.).

The above fields are also known as **Header**. We suggest (but not require) you to create a struct **Header** to include them.

6. data/content - the actual transactions carried by this block. We suggest to use a **Vec** of **Transaction**. Transaction struct is the one you wrote in previous assignment.

We suggest (but not require) you to create a struct **Content** to include the content.

Notice that to create the Merkle root of **Transaction**, you need to implement trait **Hashable** for **Transaction**. The way to implement that trait is first serialize it into bytes, then call SHA256 to hash the bytes.

You need to implement trait **Hashable** for **Block**. They way to hash a block is to hash **Header** rather than **Content**. So you can first implement **Hashable** for **Header**. When you hash a **Block**, you can directly call the hash function of **Header**.

To test and debug, you need to finish the function *generate_random_block*. This function takes an argument named *parent*. The generated block should contain that *parent*. And the *nonce* should be a random integer. As for content, you can simply let it be empty. So merkle\_root should be the Merkle root of an empty input. As for fields such as difficulty and timestamp, choose whatever you like.

### Blockchain

You need to finish a struct named **Blockchain**, which contains the necessary information of a direct acyclic graph (DAG) and provides functions related to the longest chain rule. The following functions are required:
1. new() - create a new blockchain that only contains the information of the genesis block. (Define genesis block by your self.)
2. insert() - insert a block into the blockchain.
3. tip() - return the last block's hash in the longest chain.
4. all_blocks_in_longest_chain() - return all blocks' hashes, from the genesis to the tip. This function will not be tested in this part, and will be used in debugging in the future.

#### Storage choice

We suggest that you use a **HashMap** in standard crate to store blocks. You can use hash as key and the block as value. It can look up for blocks by hash very conveniently.

You can also store the tip, and update it after inserting a block. If, say, your current tip is hash(B1), and you insert a new block B2. You need to update tip to hash(B2) if and only if the length of chain B2 is *strictly greater* than that of B1.

Since we are following the longest chain rule, you may also store the length/height of each block. And use the height to determine the longest chain. E.g., genensis block has height 0.

You may worry that the client will run out of memory. You can implement with persistent storage such as database, but this is not the point of this project and we suggest to just use in-memory storage.

#### Thread safety choice

In the future, the **Blockchain** struct will be shared between threads, such as miner thread and network thread. So this struct must be thread safe. However, this is not hard to do with lock. **You don't need to worry about it in this part.** You can implement a non-thread-safe **Blockchain** and leave the thread safety problem to future parts.

## Grading

If your programming is working, you will pass the test named *insert_one*. (By running `cargo test`.)

We will use other private tests to grade your submission.
The tests will insert several blocks into a new blockchain, and check whether the tip is correct. The tests contain forking/branching scenarios to check the correctness of your longest chain rule.

We will *NOT* call insert function with invalid blocks. Specifically, we will not insert a block whose parent is not already inserted.

## Advance Notice
1. If you want to learn about thread safety of the Blockchain struct, you can try `Arc<Mutex<Blockchain>>` in your code.
2. Our goal is to decouple blockchain status from ledger status, and focus on the former. As a result, we don't involve transaction execution or ledger update or UTXO in this part. They will be handled in future parts.
3. We don't involve proof-of-work check or mining, but we need to prepare for them. So we require fields nonce and difficulty inside a block. You can start to think about how to mine or check blocks.
4. Blockchain struct will be used in multiple places in the future. For example, when you implement a miner, you insert a mined block into blockchain; when you want to mine on the longest chain, you need to get the tip as the block's parent; when you receive a block from p2p network, you insert it.
5. (Same as in assignment 2.) A tricky part about transaction and signature is how you put them together. One possible way is to create another struct called SignedTransaction. The other way is to declare a field in transaction called signature, which will be empty if there is no signature. Feel free to design your own way. Obviously, a block should carry transactions *with* signature in order to verify it. (We don’t require you to finish this in this part.)
6. Updated: we don't require you to put a coin base transaction inside blocks in this part.
