"""동시에 N개까지만 시작시키는 다운로드 대기열. Qt 없이 동작한다."""

from __future__ import annotations

from typing import Callable, Generic, TypeVar

T = TypeVar("T")


class JobQueue(Generic[T]):
    def __init__(self, limit: int, start: Callable[[T], None]):
        self.limit = max(1, limit)
        self._start = start
        self._running: list[T] = []
        self._pending: list[T] = []

    @property
    def running(self) -> list[T]:
        return list(self._running)

    @property
    def pending(self) -> list[T]:
        return list(self._pending)

    def add(self, job: T) -> None:
        self._pending.append(job)
        self._fill()

    def done(self, job: T) -> None:
        if job in self._running:
            self._running.remove(job)
            self._fill()

    def set_limit(self, n: int) -> None:
        # 줄여도 이미 돌던 작업은 끝까지 간다. 빈 자리가 한도 아래로 내려갈 때까지 새로 시작하지 않는다.
        self.limit = max(1, n)
        self._fill()

    def _fill(self) -> None:
        while self._pending and len(self._running) < self.limit:
            job = self._pending.pop(0)
            self._running.append(job)
            self._start(job)
