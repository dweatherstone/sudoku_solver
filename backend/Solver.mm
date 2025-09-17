<map version="freeplane 1.7.0">
<!--To view this file, download free mind mapping software Freeplane from http://freeplane.sourceforge.net -->
<node TEXT="Solver" FOLDED="false" ID="ID_700589549" CREATED="1757939597355" MODIFIED="1757939608998" STYLE="oval">
<font SIZE="18"/>
<hook NAME="MapStyle">
    <properties edgeColorConfiguration="#808080ff,#ff0000ff,#0000ffff,#00ff00ff,#ff00ffff,#00ffffff,#7c0000ff,#00007cff,#007c00ff,#7c007cff,#007c7cff,#7c7c00ff" fit_to_viewport="false"/>

<map_styles>
<stylenode LOCALIZED_TEXT="styles.root_node" STYLE="oval" UNIFORM_SHAPE="true" VGAP_QUANTITY="24.0 pt">
<font SIZE="24"/>
<stylenode LOCALIZED_TEXT="styles.predefined" POSITION="right" STYLE="bubble">
<stylenode LOCALIZED_TEXT="default" ICON_SIZE="12.0 pt" COLOR="#000000" STYLE="fork">
<font NAME="SansSerif" SIZE="10" BOLD="false" ITALIC="false"/>
</stylenode>
<stylenode LOCALIZED_TEXT="defaultstyle.details"/>
<stylenode LOCALIZED_TEXT="defaultstyle.attributes">
<font SIZE="9"/>
</stylenode>
<stylenode LOCALIZED_TEXT="defaultstyle.note" COLOR="#000000" BACKGROUND_COLOR="#ffffff" TEXT_ALIGN="LEFT"/>
<stylenode LOCALIZED_TEXT="defaultstyle.floating">
<edge STYLE="hide_edge"/>
<cloud COLOR="#f0f0f0" SHAPE="ROUND_RECT"/>
</stylenode>
</stylenode>
<stylenode LOCALIZED_TEXT="styles.user-defined" POSITION="right" STYLE="bubble">
<stylenode LOCALIZED_TEXT="styles.topic" COLOR="#18898b" STYLE="fork">
<font NAME="Liberation Sans" SIZE="10" BOLD="true"/>
</stylenode>
<stylenode LOCALIZED_TEXT="styles.subtopic" COLOR="#cc3300" STYLE="fork">
<font NAME="Liberation Sans" SIZE="10" BOLD="true"/>
</stylenode>
<stylenode LOCALIZED_TEXT="styles.subsubtopic" COLOR="#669900">
<font NAME="Liberation Sans" SIZE="10" BOLD="true"/>
</stylenode>
<stylenode LOCALIZED_TEXT="styles.important">
<icon BUILTIN="yes"/>
</stylenode>
</stylenode>
<stylenode LOCALIZED_TEXT="styles.AutomaticLayout" POSITION="right" STYLE="bubble">
<stylenode LOCALIZED_TEXT="AutomaticLayout.level.root" COLOR="#000000" STYLE="oval" SHAPE_HORIZONTAL_MARGIN="10.0 pt" SHAPE_VERTICAL_MARGIN="10.0 pt">
<font SIZE="18"/>
</stylenode>
<stylenode LOCALIZED_TEXT="AutomaticLayout.level,1" COLOR="#0033ff">
<font SIZE="16"/>
</stylenode>
<stylenode LOCALIZED_TEXT="AutomaticLayout.level,2" COLOR="#00b439">
<font SIZE="14"/>
</stylenode>
<stylenode LOCALIZED_TEXT="AutomaticLayout.level,3" COLOR="#990000">
<font SIZE="12"/>
</stylenode>
<stylenode LOCALIZED_TEXT="AutomaticLayout.level,4" COLOR="#111111">
<font SIZE="10"/>
</stylenode>
<stylenode LOCALIZED_TEXT="AutomaticLayout.level,5"/>
<stylenode LOCALIZED_TEXT="AutomaticLayout.level,6"/>
<stylenode LOCALIZED_TEXT="AutomaticLayout.level,7"/>
<stylenode LOCALIZED_TEXT="AutomaticLayout.level,8"/>
<stylenode LOCALIZED_TEXT="AutomaticLayout.level,9"/>
<stylenode LOCALIZED_TEXT="AutomaticLayout.level,10"/>
<stylenode LOCALIZED_TEXT="AutomaticLayout.level,11"/>
</stylenode>
</stylenode>
</map_styles>
</hook>
<hook NAME="AutomaticEdgeColor" COUNTER="4" RULE="ON_BRANCH_CREATION"/>
<node TEXT="1. Find Most Constrained Cell" POSITION="right" ID="ID_391829802" CREATED="1757939611551" MODIFIED="1757940080018">
<edge COLOR="#ff0000"/>
<node ID="ID_129883047" CREATED="1757940143283" MODIFIED="1757940154266"><richcontent TYPE="NODE">

<html>
  <head>
    
  </head>
  <body>
    <p>
      Returns <b>NextCell</b>
    </p>
  </body>
</html>

</richcontent>
<node ID="ID_1964586540" CREATED="1757940154270" MODIFIED="1757940186647"><richcontent TYPE="NODE">

<html>
  <head>
    
  </head>
  <body>
    <p>
      <b>Cell: </b>If there is a next cell to find
    </p>
  </body>
</html>

</richcontent>
</node>
<node ID="ID_142103471" CREATED="1757940187015" MODIFIED="1757940210010"><richcontent TYPE="NODE">

<html>
  <head>
    
  </head>
  <body>
    <p>
      <b>NoEmptyCells:</b>&#160;Nothing to fill -&gt; calls validate_solution
    </p>
  </body>
</html>

</richcontent>
</node>
<node ID="ID_1405069662" CREATED="1757940210229" MODIFIED="1757940252598"><richcontent TYPE="NODE">

<html>
  <head>
    
  </head>
  <body>
    <p>
      <b>DeadEnd:</b>&#160;Usually if there is a cell with no possibilities -&gt; solution cannot be found so rollback
    </p>
  </body>
</html>

</richcontent>
</node>
</node>
<node TEXT="1. Loop through solver.possibilities" ID="ID_882909685" CREATED="1757940265661" MODIFIED="1757940595418">
<node ID="ID_1414491439" CREATED="1757940595420" MODIFIED="1757940612878"><richcontent TYPE="NODE">

<html>
  <head>
    
  </head>
  <body>
    <p>
      <b>If empty</b>&#160;-&gt; Return NextCell::DeadEnd
    </p>
  </body>
</html>

</richcontent>
</node>
</node>
</node>
<node TEXT="2. Update Possibilities" POSITION="left" ID="ID_583344531" CREATED="1757942455383" MODIFIED="1757942467216">
<edge COLOR="#ff00ff"/>
<node TEXT="Loop through all cells" ID="ID_1986610365" CREATED="1757942477530" MODIFIED="1757942491320"/>
<node TEXT="If the cell is empty (== 0)" ID="ID_364670199" CREATED="1757942519622" MODIFIED="1757942542691">
<node TEXT="1. Get standard possibilities" ID="ID_486758906" CREATED="1757942544584" MODIFIED="1757942556030"/>
<node TEXT="2. Get possibilities for all variants" ID="ID_1017723137" CREATED="1757942557216" MODIFIED="1757942581409"/>
<node TEXT="Insert into solver.possibilities the intersection of the previous 2 steps" ID="ID_817264601" CREATED="1757942586965" MODIFIED="1757942616085"/>
</node>
<node TEXT="If the cell is not empty (1..=9)" ID="ID_424079134" CREATED="1757942620557" MODIFIED="1757942640569">
<node TEXT="Remove cell from solver.possibilities" ID="ID_521919330" CREATED="1757942642155" MODIFIED="1757942651749"/>
</node>
</node>
</node>
</map>
