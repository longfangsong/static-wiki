---
category: concept
tags: [transaction, TiKV, Lock]
---
# LockManager

TiKV中用于管理悲观锁的组件。

## 辨析

[WaiterManager](/zh/what/WaiterManager.html) vs [LockManager](/zh/what/LockManager.html): [WaiterManager](/zh/what/WaiterManager.html) 是 [LockManager](/zh/what/LockManager.html) 的一部分，[LockManager](/zh/what/LockManager.html) 同时还有另一部分 [Detector](/zh/what/Detector.html) 负责进行死锁检测。

## Links

- [代码](https://github.com/tikv/tikv/blob/2a2fa03da53b63f3fc24d7ea53aead40176979b5/src/storage/lock_manager.rs#L48)
