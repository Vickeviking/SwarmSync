# Overview of system Architecture of swarm-sync

Swarm sync is a system allowing a user to run jobs on a swarm of machines.
Consumer -> core -> worker -> core -> consumer

The system is a docker swarm with 3 services: core, worker and consumer
Allowing for linux built docker containers to be uploaded to core, the consumer then is able
to fetch results from the core, when workers are done.

The modules are robust and secured with encryption and authentication, allowing for secure data transfer aswell as
safe routes guarded from unauthorized access. The worker runs the programs in a docker container providing a safe
environment for the program to run in, minimizing the risk of security vulnerabilities and malware.

The data stored is persistent in a postgres database, allowing for easy access and management of the data.
Aswell as caching the data in redis, allowing for faster access to the data.

Communication between the modules is done through a websocket, allowing for easy communication between the modules.

- Important and unique data is sent via Rocket web server, where PG management is done through Diesel ORM.
- Faster updates, like heartbeats from workers are sent to core through UDP.

## Technology Stack

- Docker
  All modules are built in docker containers, allowing for easy deployment and management of the modules.
- Postgres
  The database used to store the data, providing a persistent storage for the data.
- Redis
  The cache used to store the data, providing a fast access to the data.
- Rust
  Each module is built in rust, providing a secure and reliable environment for the modules to run in.
- Diesel ORM
  The ORM used to interact with the database, providing a simple and easy to use interface to the database.
- Rocket
  The web framework used to build the modules, providing a simple and easy to use interface to the modules.

---

## swarm-core

Swarm core is the main module of the system, providing the core functionality of the system.
It hosts the web interface and the API, allowing for easy management of the system.
Aswell as the data management through postgres and redis.
The Rocket webinterface and the system environment are running togheter as `Core-API`

### Core-API

The Core-API starts all the modules in an Async TOKIO runtime, allowing for easy management of the modules.
The modules:

- Dispatcher
  Handles a UDP server, allowing for dynmamic updates of worker availability.
  Sends out job that are ready to be run from `Scheduler` to the correct worker.
  Chooses worker, not job. Before sending the job to the worker it also updates
  `Harvester` on what jobs are being sent so it knows what to await and from whom.
- Harvester
  Recieves results from workers, when everything is fetched and deemed OK,
  it sends the result to `TaskArchive`.
- Hibernator
  After `Reciever` recieves a job from Consumer, it is either sent to `Scheduler` or to `Hibernator`.
  If it is not meant to be ran now, but instead holds a cron expression, it is sent to `Hibernator`.
  Here it sleeps for the correct time and then sends the job to `Scheduler`.
- Logger
  A struct protected by a mutex, shared between all of the modules.
  When modules need to log something they can log through the logger.
  Logger provides a rich interface with tags, filters and custom logging levels.
- Reciever
  Recieves jobs from the consumer, and sends them to the correct module.
- Scheduler
  Using a non-preemptive scheduling algorithm, it picks the next job to be sent to a worker.
  `Dispatcher` reads this job when its ready and sends it to the worker.
- TaskArchive
  A module that keeps track of the results of the jobs.
  Consumer can fetch the results from this module.
  In Future this will be deprecated, this since the meaning of this module
  seems to be unclear since we already have a postgres database for storing the results.

### Commanddeck

The TUI interface for the Core-API, allowing for easy management of the system.
CRUD operation of most of the data, worker configuration and system configuration.
As well as th ability to monitor logs, modules and jobs/worker interactions.

## swarm-consumer

Swarm consumer is the module that is used by the user to upload jobs to the system.
It is built in rust and talks to the core via the Core-API rocket webserver.

## swarm-worker

Swarm worker is the module that is used by the system to run jobs.
It is built in rust and talks to the core via the Core-API rocket webserver.
Sends UDP heartbeats to the core, and recieves jobs from the core.
