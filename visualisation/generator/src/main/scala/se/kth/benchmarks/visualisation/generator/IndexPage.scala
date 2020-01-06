package se.kth.benchmarks.visualisation.generator

import scalatags.Text.all._
import scalatags.generic.Attr

case class IndexPage(plots: List[Plot]) {
  def generate(): String = "<!DOCTYPE html>" + page.render;

  val integrity: Attr = Attr("integrity");
  val crossorigin: Attr = Attr("crossorigin");

  lazy val page = html(
    head(
      scalatags.Text.tags2.title("MPP Suite Index"),
      link(rel := "stylesheet", `type` := "text/css", href := "bootstrap.min.css"),
      link(rel := "stylesheet", `type` := "text/css", href := "main.css"),
      link(rel := "stylesheet", `type` := "text/css", href := "standard.css"),
      script(src := "https://code.jquery.com/jquery-1.12.4.min.js",
             integrity := "sha256-ZosEbRLbNQzLpnKIkEdrPv7lOy9C27hHQ+Xp8a4MxAQ=",
             crossorigin := "anonymous"),
      script(src := "https://code.highcharts.com/highcharts.js", crossorigin := "anonymous"),
      script(src := "https://code.highcharts.com/modules/data.js", crossorigin := "anonymous"),
      script(src := "benchmark-suite-plotting.js")
    ),
    body(
      h1(StandardStyle.headline, "MPP Suite Experiment Run"),
      h2("Experiments"),
      ul(
        for (plot <- plots) yield li(a(href := plot.relativePath, plot.title))
      ),
      div(id := "container"),
      pre(
        id := "csv1",
        StandardStyle.hidden,
        """Month, Series
      0,7.0
      1,6.9
      2,9.5
      3,14.5
      4,18.4
      5,21.5
      6,25.2
      7,26.5
      8,23.3
      9,18.3
      10,13.9
      11,9.6"""
      ),
      script(
        raw("""
        Plotting.plot("Fruit Consumptin", "Units", document.getElementById('csv1'), document.getElementById('container'));
        // $('#container').highcharts({
        //   chart: {
        //     type: 'column'
        //   },
        //   data: {
        //     csv: document.getElementById('csv1').innerHTML
        //   },
        //   title: {
        //     text: 'Fruit Consumption'
        //   },
        //   yAxis: {
        //     title: {
        //       text: 'Units'
        //     }
        //   }
        // });
        """)
      )
    )
  );
}