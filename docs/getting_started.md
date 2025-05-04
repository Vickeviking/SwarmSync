# Setting up swarm-sync

## 1. Core server

First a server is needed, this is the first thing to do.
this can be done through root folder with two make commands,

- $make up-core
- $make run-core
  This will default run the server and api. To interact with the server
  run health-checks, system overviews, aswell as CRUD operations run commanddeck.
  this is done with :
- $make run-commanddeck
  Remember this can only be done after a server is up and running,
  commanddeck needs to run on the same machine as the server in contuary to
  ´swarm-worker-tui´ & ´swarm-consumer´

## 2. Set up a User account through swarm-consumer

After spinning up the core, you are now ready to create a user account,
again through top root dir, run

- $make up-consumer
- $make run-consumer
  Now select remote or local based on where you started core server,
  then create an account by register, all passwords are encrypted so
  pick one of your choice.

## 3. Start a worker with swarm-worker

Start the worker from top dir with make commands:

- $make run-worker-tui
  Now login with the user you previously created in the consumer module.
  You now first need to add a worker, workers are created inside the
  commanddeck check above. This is found under manage Job/Worker/.. Create worker.
  After a worker is created, check for its ID under list workers.
  Now go back to swarm-worker-tui and configure a worker with this ID.
  You can now successfully Start a worker, you can also browse its logs in `browse logs`.

## 4. Start jobs

You are now ready to submit jobs inside swarm-consumer, it will be sent to core,
distributed to online Idle worker to be sent back to core. After its sent back to core you will be able
to fetch the result through the worker-consumer.
