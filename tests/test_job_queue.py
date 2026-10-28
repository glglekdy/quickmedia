from ytdl.job_queue import JobQueue


def make(limit):
    started = []
    return JobQueue(limit, started.append), started


def test_starts_up_to_limit_in_order():
    q, started = make(2)
    for j in "abcd":
        q.add(j)
    assert started == ["a", "b"] and q.pending == ["c", "d"]


def test_done_starts_next():
    q, started = make(1)
    q.add("a"); q.add("b")
    q.done("a")
    assert started == ["a", "b"] and q.running == ["b"] and q.pending == []
    q.done("zzz")  # 모르는 작업
    assert q.running == ["b"]


def test_raise_limit_starts_immediately():
    q, started = make(1)
    for j in "abc":
        q.add(j)
    q.set_limit(3)
    assert started == ["a", "b", "c"]


def test_lower_limit_keeps_running_jobs():
    q, started = make(3)
    for j in "abcd":
        q.add(j)
    q.set_limit(1)
    assert q.running == ["a", "b", "c"]
    q.done("a"); q.done("b")
    assert started == ["a", "b", "c"]  # 아직 한도(1)를 넘는다
    q.done("c")
    assert started == ["a", "b", "c", "d"]
