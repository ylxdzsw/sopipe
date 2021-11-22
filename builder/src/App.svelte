<script>
      import { onMount } from 'svelte';
  import Blockly from 'blockly';
  import DarkTheme from '@blockly/theme-dark';
    import load_blocks from './blocks';
    import codegen from './codegen';
    import patch from './intertion_marker_manager_monkey_patch'

  patch(Blockly)
    load_blocks(Blockly)
  onMount(() => {
    Blockly.inject(document.getElementById("blocklyDiv"), {
        theme: DarkTheme,
      toolbox: document.getElementById("toolbox")
    });

    Blockly.getMainWorkspace().addChangeListener(e => {
        var code = codegen(this);
    document.getElementById('output').textContent = code;
    });

    window.B = Blockly; // for debugging
  });
</script>

<style>
  #app {
    font-family: "Avenir", Helvetica, Arial, sans-serif;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    text-align: center;
    color: #2c3e50;
  }
  #blocklyDiv {
    height: 100%;
    width: 80%;
    position: absolute;
    bottom: 0;
    text-align: left;
  }
  #output {
    height: 100%;
    width: 20%;
    position: absolute;
    bottom: 0;
    right: 0;
    overflow-y: auto;
  }
</style>

<div id="app">
    <div id="blocklyDiv" />
  <xml id="toolbox" style="display:none">
    <category name="Endpoints">
        <block type="tcp" />
    </category>

    <category name="Control">
        <block type="sucks" />
        <block type="controls_ifelse" />
        <block type="logic_compare" />
        <block type="logic_operation" />
        <block type="controls_repeat_ext">
        <value name="TIMES">
            <shadow type="math_number">
            <field name="NUM">10</field>
            </shadow>
        </value>
        </block>
    </category>
    <!-- <category name="Control">
        <block type="logic_operation" />
        <block type="logic_negate" />
        <block type="logic_boolean" />
        <block type="logic_null" disabled="true" />
        <block type="logic_ternary" />
        <block type="text_charAt">
        <value name="VALUE">
            <block type="variables_get">
            <field name="VAR">text</field>
            </block>
        </value>
        </block>
    </category> -->
  </xml>
  <div id="output">

  </div>
</div>
