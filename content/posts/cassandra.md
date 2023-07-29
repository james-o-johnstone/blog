---
title: "What you should understand before using Apache Cassandra in production"
date: 2019-07-27T08:53:17+01:00
summary: "An overview of Apache Cassandra compaction, and how to correctly set disk space alerts when self-managing a production Cassandra cluster."
draft: false
---

Cassandra is a NoSQL database, the 'right choice when you need scalability and high availability without compromising performance'. It offers great performance, but self managing a Cassandra cluster really requires a deep understanding of how it works to avoid running into issues.

Each table in Cassandra has a memtable and an SSTable (Sorted Strings Table).

A memtable is an in-memory data-structure which, alongside a commit log, stores writes temporarily. The commit log can be used in the event of a crash (e.g. a power failure) to replay commits and rebuild the memtable. Keeping stuff in memory means there is no disk IO needed for writes, so these are much faster.

When the commit log approaches its maximum size, or the memtable size exceeds a ([configurable](https://docs.datastax.com/en/archived/cassandra/3.0/cassandra/operations/opsMemtableThruput.html)) threshold, memtables are flushed to disk as SSTables. At this point the memtable is cleared and the commit log is truncated.

SSTables are immutable key-value mappings, where keys and values are arbitrary byte strings. Oh and the keys are sorted too (hence the name: "Sorted String table"). The important point to remember here is that they are **immutable**. This means that there's no way to change what's in there without recreating the whole thing.

So once this SSTable has been created, the memtable has been cleared, the commit log has been truncated - the process starts over - a new memtable is written-to until being flushed to another SSTable.

If there are many write operations occuring then there are going to be many SSTables, and if there are a lot of updates/deletions happening then these SSTables could get large and be filled with tombstones and intermediate state. When reading data there's no point in storing this intermediate state, no point in storing deleted rows any more, so there's another process that merges SSTables together into a single table, "summarising" the data so that it reflects the final state. This process is called compaction.

In compaction, the newest values for updated keys are written to a new SSTable, and deleted keys aren't carried over to the new SSTable.

Compaction requires disk space - enough disk space to store the old data, and the new data, at the same time. If there's not enough space then compaction can't occur.

This is where you can run into trouble without realising it. You might set a free disk alert to fire when disk space is > 90% for most cases, but with Cassandra, this alert needs to occur before 50% disk usage, because once you have surpassed that threshold, compactions can no longer occur. Depending on which Cassandra version you are on, they can also fail silently.

If you get to 90% disk then there is no way of freeing up disk space at that point - because compactions need to run to create space, but they can't. You could add another node to increase storage of the cluster as a whole, but then there will be a large period of time whilst that node catches up (whilst data is streamed accross to the new node), and you will still need to run `nodetool cleanup` on the old nodes, which is going to require 50% disk!

At this point, you'll have to take down the nodes, add bigger disks, copy the data over and restart them - this is going to take some time, if you are running a busy production cluster then that's not ideal!
