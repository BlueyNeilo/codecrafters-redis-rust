This is a starting point for Rust solutions to the
["Build Your Own Redis" Challenge](https://codecrafters.io/challenges/redis).

In this challenge, you'll build a toy Redis clone that's capable of handling
basic commands like `PING`, `SET` and `GET`. Along the way we'll learn about
event loops, the Redis protocol and more.

**Note**: If you're viewing this repo on GitHub, head over to
[codecrafters.io](https://codecrafters.io) to try the challenge.

# Passing the first stage

The entry point for your Redis implementation is in `src/main.rs`. Study and
uncomment the relevant code, and push your changes to pass the first stage:

```sh
git add .
git commit -m "pass 1st stage" # any msg
git push origin master
```

That's all!

# Stage 2 & beyond

Note: This section is for stages 2 and beyond.

1. Ensure you have `cargo (1.54)` installed locally
1. Run `./spawn_redis_server.sh` to run your Redis server, which is implemented
   in `src/main.rs`. This command compiles your Rust project, so it might be
   slow the first time you run it. Subsequent runs will be fast.
1. Commit your changes and run `git push origin master` to submit your solution
   to CodeCrafters. Test output will be streamed to your terminal.


# My Notes

The final solution to the codecrafters challenge is in branch `codecrafters-stage7`.

In the main branch, I am solving the extra challenges using different dependencies which are incompatible with codecrafters CI.

## Extra Challenges Planned

- [ ] Write tests in rust that prove thread safety
- [ ] Persistence + Recovery (RDB snapshot and/or Append-Only File)
- [ ] High Availability communication + failover (Keep replica server in sync, replication id/offset)
- [ ] Authentication
- [ ] [Garbage collection on long-lived expired keys](https://redis.io/commands/expire/)
- [ ] Handle other data types like sets/hashes

## Additional References

- [Redis Architecture](https://architecturenotes.co/redis/)
- [Redis Command Docs](https://redis.io/commands/)
