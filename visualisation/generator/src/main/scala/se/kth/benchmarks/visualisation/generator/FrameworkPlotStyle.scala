package se.kth.benchmarks.visualisation.generator


object FrameworkPlotStyle {
  val Colors:Map[String, String] = Map(
    "'Akka'"-> "'#18A8CE'",  // Akka Teal
    "'Akka Typed'" -> "'#085567'", // Akka Cyan
    "'Erlang'" -> "'#A90433'", // Erlang Red
    "'Kompact Actor'" -> "'Black'",
    "'Kompact Component'" -> "'DarkGrey'",
    "'Kompact Mix'" -> "'SaddleBrown'",
    "'Kompics Java'" -> "'DarkOrange'",
    "'Kompics Scala 1.x'" -> "'Violet'",
    "'Kompics Scala 2.x'" -> "'DeepPink'",
    "'Riker'" -> "'#2F0272'", // Riker Purple
    "'Actix'" -> "'Green'", // Too many blue colors : (
  );

  def getColor(framework: String): String = {
    val default_color = "'Yellow'"
    if (framework.contains(" error") ) {
      val f = framework.substring(0, framework.indexOf(" error"));
      println(f);
      Colors.getOrElse(f+"'", default_color)
    } else {
      Colors.getOrElse(framework, default_color)
    }
  }
}
