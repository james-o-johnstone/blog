---
title: "Java Performance"
date: 2023-07-29:33:00+01:00
draft: false
---

Over the last 2 years, I have been focused on improving the performance of the various systems that I work on and levelling up my Java skills. As part of this focus I read through the [Java Performance](https://www.oreilly.com/library/view/java-performance-2nd/9781492056102/) O'Reilly book. To cement my learning and help anyone who stumbles across this post learn something new, I decided to write this post summarising the 10 best things I learned reading the book.

# 1. Prematurely optimize

The big idea is to always prematurely optimize by ensuring that on every line of code you are thinking about not introducing badly performing code. For example, rather than checking whether something exists in a list of string, convert the list to a set so that each lookup is O(1) rather than O(N). This ensures that your applications do not perform badly as a result of death by 1000 cuts. This approach doesn't mean overengineering to get better performance but avoiding death by 1000 cuts by thinking about performance constantly whilst writing and reviewing code.

# 2. The database is always the bottleneck

There is no point spending time optimising or tuning an application when the database is the bottleneck, i.e. investing time in rearchitecting/tuning java code to gain a few cpu cycles is pretty pointless if the database is already saturated. The point is a bit dogmatic but the general principle is to analyze the performance of the system holistically, and if the database is already overloaded then making the java application more efficient is going to make the problem worse.

# 3. Test Early, Test Often

I like the idea of performance testing as a part of the development cycle and getting instant feedback on whether changes have caused a performance regression. In my current role I introduced nightly load tests that run at the same time every night on the code currently deployed in staging and production. The tests generate statistics on latency quantiles (P50, P90, P99, P99.9) and plots that developers can look at after they deployed some code and easily compare with previous tests to see whether their changes affected the performance of the system.

# 4. Using Less Memory

Similar to point 1, this point is around choosing smaller data types for instance variables to avoid a death by 1000 cuts scenario with memory usage. Examples:
- Using a byte to keep track of states if there are only 8 states
- Using float instead of double
- Using int instead of long

# 5. Synchronization

Synchronization has performance overheads: firstly if a lock is contended then a thread must wait for the lock to be released, secondly when a thread leaves a synchronized block it must flush any modified variables to main memory so that other threads can see updated values. This is an expensive operation called register flushing (a register is a fast memory location on a specific CPU), and the Java Memory Model requires this flushing so that all data from one thread becomes visible to another thread that synchronizes on the same lock.

To get around the performance overheads of synchronization, consider avoiding synchronization altogether by restructuring code using `ThreadLocal` variables or CAS-based (Compare and Swap) alternatives e.g. `AtomicLong`, `AtomicReference` etc.

Some important things to note about CAS:
- If access to a resource is uncontended, CAS-based alternatives will be slightly faster than synchronization.
- If access to a resource is lightly/moderately contended, CAS-based protection will be faster than synchronization.
- If access to a resource is heavily contended, synchronization will become a more efficient choice at some point.
- There is no contention when values are read and not written.

# 6. Prepared Statements

It is better to reuse `PreparedStatements` within an application than creating `Statements` for database calls. Prepared statements allow the database to process the SQL more efficiently, there is an overhead when first creating a `PreparedStatement` so they should be reused.

# 7. JSON Processing

It is better to create a single `ObjectMapper` in an application and share between classes rather than creating a separate `ObjectMapper` per class. This can reduce memory pressure, excessive GC cycles and CPU usage.

# 8. String Concatenation

One line concatenation using `+` yields good performance, e.g. `String s = "hello" + " there";`

For multiple concatenation operations, use `StringBuilder`, e.g.
```
StringBuilder sb = new StringBuilder();
for (int i = 0; i < nStrings; i++) {
    sb.append(strings[i]);
}
```

performs better than
```
String s = "";
for (int i = 0; i < nStrings; i++) {
    s = s + strings[i];
}
```

# 9. BufferedIO

For IO it's better to use the `Buffered` classes e.g. `BufferReader`/`BufferedWriter` for string data. This is because the unbuffered classes read/write a single byte at a time which requires a round trip to the kernel for every operation. Using the buffered classes ensures that the data is buffered in the JVM and read/written in batches rather than single operations.

# 10. GC Algorithms

I always though G1GC was the one and only GC algorithm to use, but actually there are usecases for using other GC algorithms and I found the overview of GC algorithms in this book really useful.

## Quick overview of GC alogrithms:

GC algorithms find unused objects, free the memory they are using and then compact that free memory to prevent memory fragmentation. In order to make the process more efficient, the heap is split into the old generation and young generations. When objects are first created they're allocated in the young generation, and moved to the old generation if they survive a GC cycle. Splitting the heap means that GC can be split into two operations: minor GC and full GC. Minor GC only needs to collect garbage for a small portion of the heap so it can run faster and more often.

The specific GC algorithm will balance and perform the above operations slightly differently, and choosing the correct algorithm is a trade off.

## Serial Collector

Is a single threaded GC that stops all other threads when running both minor and full GC. It is the default when there is 1 CPU.

## Parallel/Throughput Collector

It is a multi-threaded GC that stops all threads in both minor and full GC. It is the default in JDK 8 when there are >= 2 CPUs.

## G1GC (Garbage First Garbage Collector)

It is a concurrent, multi-threaded GC that stops all threads when collecting the young generation but is able to process the old generation in the background without needing to stop application threads. Default in JDK 11 and later when there are >= 2 CPUs

## CMS Collector

It is a concurrent, multi-threaded GC that stops all threads during a minor GC, and (like G1GC) is also able to process the old generation in the background. Unlike G1GC it must stop the world in order to compact the heap if it becomes fragmented.
