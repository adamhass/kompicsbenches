package se.kth.benchmarks.kompicsscala

import org.scalatest._

class LocalTest extends FunSuite with Matchers {
  test("Local communication") {
    val ltest = new se.kth.benchmarks.test.LocalTest(new BenchmarkRunnerImpl());
    ltest.test();
  }
}
