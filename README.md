# THEIA: scheduling resource tenants on logical infrastructure

Hypothetically, you are running a significant infrastructure with multiple data centres. 
Let's assume that you are interested in partitioning your total infrastructure in chunks for customers (yes very cloud-like).
This works fine, if you have 'near-infinite resources' like typical hyper-scalers do, but generally you will not be able to achieve this level of autonomy.
`THEIA` aims to help in scheduling these chunks (better known as tenants), on an infrastructural level, not assuming implementation details (like running specific partitionable software, ex. K8s).

## Scope

- `THEIA` assumes that all infrastructure has 3 components (and each data centre exposes information about):
  1. compute
  2. storage
  3. networking
- `THEIA` 
- 

## Installation

In order for `THEIA` to function, there needs to be at least 1 instance running per management plane of a data centre exposing resources.
There need to be services running in the management plane, that can be queried to:
- sum the total amount of compute, storage and networking available
- sum the total free amount of these resources
- (re-)assign resources to a given tenant. 

### Some more details later

## Flow

```
```

## Audit trails

One of the main goals of `THEIA` is to create transparency in scheduling. 
We want to know which resources are claimed, by whom and when, in order to optimally serve them.
`THEIA` logs the flow, exposes a configurable amount of history of it via a 'shielded' API call, and has connectors for:
- [Elastic APM](https://www.elastic.co/observability/application-performance-monitoring)
- [Sentry](https://sentry.io/welcome/)
- [Loki](https://grafana.com/oss/loki/)

in order to send traces and events to external monitoring systems.

## Distributed storage

`THEIA` uses [Redis](https://redis.io) as its synchronization backend to allow for multiple application servers. 
During initial start of the first `THEIA` instance, you will need to generate a single secret, which is used for encrypting data stored in Redis. 
It is recommended to generate this random the following way:

```
dd if=/dev/urandom bs=32 count=1 2>/dev/null | sha256sum -b | sed 's/ .*//'
```

This secret needs to be available in one of 3 places (the lower the number, the higher the precedence in case of multiple secrets being configured):
1. as a parameter passed to the application (--secret)
2. as an environment variable (THEIA_SECRET=)
3. as a configuration parameter in the configuration file (SECRET=)

The Redis connection URI needs to be available in a similar manner:
1. as a parameter passed to the application (--redis)
2. as an environment variable (THEIA_REDIS=)
3. as a configuration parameter in the configuration file (REDIS=)

## Configuration file

We look for a configuration file in the following spots (and again the lower the number in this list, the higher the precedence):
1. custom: as a parameter passed to the application (--config)
2. user: ~/.config/theia/config.yaml
3. system: /etc/theia/config.yaml

***Note*** if partial information exists in multiple files the precedence defines which value is actually chosen.

```
if 2 configuration files exist, system and user, with the following contents:

system
  secret: abcd
  redis: redis://localhost

user
  secret: efgh
  ntp: check

the resulting configuration would be:

runtime
  secret: efgh
  ntp: check
  redis: redis://localhost
```


