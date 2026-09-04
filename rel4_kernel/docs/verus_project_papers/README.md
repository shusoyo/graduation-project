# Verus Projects 对应论文整理

来源页面：

- <https://verus-lang.github.io/verus/publications-and-projects/>

本目录收集了该页面 **Projects** 部分提到的项目所明确关联、或在项目仓库中直接引用的论文 PDF。

## 项目与论文对应关系

### 1. A Distributed Key-Value Store

项目仓库：

- <https://github.com/verus-lang/verified-ironkv>

关联论文：

- `pdfs/01-ironfleet-proving-practical-distributed-systems-correct-sosp2015.pdf`
  - IronKV 仓库 README 明确说明该项目是对 IronFleet 中 IronSHT 系统的移植，因此这里收录其直接来源论文。

### 2. A Concurrent Memory Allocator

项目仓库：

- <https://github.com/verus-lang/verified-memory-allocator>

关联论文：

- `pdfs/02-verus-verifying-rust-programs-using-linear-ghost-types-oopsla2023.pdf`
  - 该仓库 README 直接链接到这篇 Verus 设计论文。

### 3. A Node Replication (NR) Library

项目仓库：

- <https://github.com/verus-lang/verified-node-replication>

关联论文：

- `pdfs/03-verus-a-practical-foundation-for-systems-verification-sosp2024.pdf`
  - 仓库 benchmark README 明确提到该项目用于论文 *Verus: A Practical Foundation for Systems Verification* 的评测。

- `pdfs/04-sharding-the-state-machine-ironsync-osdi2023.pdf`
  - 仓库 README 明确说明这是 IronSync 的 Node Replication proofs 移植，因此收录 IronSync 对应论文。

### 4. Persistent Memory Storage Systems

项目仓库：

- <https://github.com/microsoft/verified-storage>

关联论文：

- `pdfs/05-power-never-corrupts-osdi2025.pdf`
  - 仓库 README 直接说明其 artifact 对应 OSDI 2025 论文 *PoWER Never Corrupts*。

### 5. An OS Page Table Management Implementation and OS Model

项目仓库：

- <https://github.com/utaal/verified-nrkernel>

状态：

- 暂未在仓库主页 README 中发现明确指向的项目论文链接。
- 仓库中有 `relatedwork/README.md`，整理了相关领域论文，但看起来更像背景资料而不是该项目本身的正式发表论文。

### 6. Asterinas OSTD (Operating System Standard Library)

项目仓库：

- <https://github.com/asterinas/vostd>

关联论文：

- `pdfs/06-cortenmm-sosp2025.pdf`
  - 仓库 README 直接链接到 SOSP 论文 *CortenMM: Efficient Memory Management with Strong Correctness Guarantees*。

### 7. A Two-Level Segregated Fit allocator

项目仓库：

- <https://github.com/unsoundsystem/rlsf-verified>

状态：

- 暂未在仓库主页 README 中发现明确指向的项目论文链接。
- 仓库文档中可以看到一些相关引用与论文链接，但没有发现“该项目自身对应的正式论文”。

## 已下载 PDF

- [01-ironfleet-proving-practical-distributed-systems-correct-sosp2015.pdf](/workspace/rel4_kernel/docs/verus_project_papers/pdfs/01-ironfleet-proving-practical-distributed-systems-correct-sosp2015.pdf)
- [02-verus-verifying-rust-programs-using-linear-ghost-types-oopsla2023.pdf](/workspace/rel4_kernel/docs/verus_project_papers/pdfs/02-verus-verifying-rust-programs-using-linear-ghost-types-oopsla2023.pdf)
- [03-verus-a-practical-foundation-for-systems-verification-sosp2024.pdf](/workspace/rel4_kernel/docs/verus_project_papers/pdfs/03-verus-a-practical-foundation-for-systems-verification-sosp2024.pdf)
- [04-sharding-the-state-machine-ironsync-osdi2023.pdf](/workspace/rel4_kernel/docs/verus_project_papers/pdfs/04-sharding-the-state-machine-ironsync-osdi2023.pdf)
- [05-power-never-corrupts-osdi2025.pdf](/workspace/rel4_kernel/docs/verus_project_papers/pdfs/05-power-never-corrupts-osdi2025.pdf)
- [06-cortenmm-sosp2025.pdf](/workspace/rel4_kernel/docs/verus_project_papers/pdfs/06-cortenmm-sosp2025.pdf)

## 原始下载链接

- IronFleet:
  - <https://www.microsoft.com/en-us/research/wp-content/uploads/2015/10/ironfleet.pdf>
- Verus OOPSLA 2023:
  - <https://www.andrew.cmu.edu/user/bparno/papers/verus-ghost.pdf>
- Verus SOSP 2024:
  - <https://www.microsoft.com/en-us/research/uploads/prod/2024/09/verus.pdf>
- IronSync / OSDI 2023:
  - <https://www.usenix.org/system/files/osdi23-hance.pdf>
- PoWER / OSDI 2025:
  - <https://www.usenix.org/system/files/osdi25-leblanc.pdf>
- CortenMM / SOSP 2025:
  - <https://web.cs.ucla.edu/~tamir/papers/sosp25.pdf>
