---
category: concept
aliases: []
tags: [TiKV]
---

# WaiterManager

处理等待悲观锁的事务的等待与唤醒。

## 辨析

[WaiterManager](/zh/what/WaiterManager.html)
vs [LockManager](/zh/what/LockManager.html): [WaiterManager](/zh/what/WaiterManager.html)
是 [LockManager](/zh/what/LockManager.html) 的一部分，[LockManager](/zh/what/LockManager.html)
同时还有另一部分 [Detector](/zh/what/Detector.html) 负责进行死锁检测。

## Links

- [代码](https://github.com/tikv/tikv/blob/515df8d552cce67111991fc6b205ec2905716c2b/src/server/lock_manager/waiter_manager.rs#L448)