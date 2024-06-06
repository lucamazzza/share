# MDFS
MDFS is a lightweight distributed file system based on clusters, that embodies all kinds of security you want to have.

## Concept
### "Distributedness":
MDFS can be distributed over your local network or even the internet by creating different clusters in different zones of the net and joining them.
The volume results as a unique one to the end user, but it is in fact separated in different zones and automatically (or manually if you wish) balanced and their errors and downtimes is automatically (or again, manually) managed.

### Storage
The storage is fixed to be to the smallest cluster connected to the system and all cluster contain the same data, as they were copied from one another.
This allows for all the clusters but one to go down and the system still working.
The filesystem poses a limit on the maximum size of a single file, which is to _be determined_.

### Downtime
If you have a big network accessing your clusters, when there are only a few remaining (normally >3) the system goes into limp mode, granting access only to a bunch of user at a time, until the downtime is fixed.

### Parallelisation
The system, on the single cluster and even between clusters, gets its efficiency on parallelisation; it means that the controller present on every cluster decides where the client is best to connect (which means the client is automatically redirected to the node more capable to manage their request, based on load and network distance).
Inside the node itself there is a parallelisation of the jobs it has to take on.

### Encryption
The whole filesystem can be encrypted using the current top notch algorithms (see version release notes for changes). The system is also able to manage permissions over files and folders.

## Installation
### Linux distributions
To install MDFS you can use your favorite package manager (see distro compatibility).

```bash
apt-get update
apt-get install mdfs
```

You have also the option to pull it directly from the repository.
```bash
curl -sL https://github.com/lucamazzza/mdfs/install/mdfs-<version>-complete.tar.gz | tar xzvf /usr/bin/
```

## Setup
To setup how the filesystem works in your system you have a set of .toml files which manage different parts and options of the system; you will find those under the folder /usr/bin/mdfs/config/.
Here is a list of the configuration files:
```bash
/usr/bin/mdfs/config
                │
                ├╴cluster.toml    // Information about the cluster and nodes (IP, route, ...)
                ├╴controller.toml // Controller policy (priority users, blacklist, ...)
                ├╴uptime.toml     // Uptime policy (max load, fallback policy, limp mode, ...)
                └ db.toml         // Database management configs (connection string, tables, ecc...)
```
