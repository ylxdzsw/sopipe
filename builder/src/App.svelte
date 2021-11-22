<script>
    import { onMount } from "svelte"
    import Blockly from "blockly"
    import DarkTheme from "@blockly/theme-dark"
    import load_blocks from "./blocks"
    import codegen from "./codegen"
    import patch from "./intertion_marker_manager_monkey_patch"

    patch(Blockly);
    const blocks = load_blocks(Blockly)
    // group by category
    const categories = blocks.reduce((rv, x) => {
        (rv[x.category] = rv[x.category] || []).push(x)
        return rv
    }, {})

    onMount(() => {
        Blockly.inject(document.getElementById("blocklyDiv"), {
            theme: DarkTheme,
            toolbox: document.getElementById("toolbox"),
        })

        Blockly.getMainWorkspace().addChangeListener((e) => {
            var code = codegen(Blockly.getMainWorkspace(), compact)
            document.getElementById("output").textContent = code
        })

        window.B = Blockly // for debugging
    })

    let compact = false
    function on_compact_clicked() {
        compact = !compact
        var code = codegen(Blockly.getMainWorkspace(), compact)
        document.getElementById("output").textContent = code
    }
</script>

<div id="app">
    <div id="blocklyDiv" />
    <xml id="toolbox" style="display:none">
        {#each Object.entries(categories) as [category, blocks]}
            <category name="{category}">
                {#each blocks as block}
                    <block type="{block.name}">
                    {#each (block.default_arg_names || []) as arg_name, i}
                        <value name="arg_{i}">
                            <shadow type="argument">
                                <field name="key">{arg_name}</field>
                                <field name="value"></field>
                            </shadow>
                        </value>
                    {/each}
                    </block>
                {/each}
            </category>
        {/each}

        <category name="Argument">
            <block type="argument" />
        </category>
    </xml>
    <div id="output" />
    <label id="compact">
        <input type="checkbox" on:change={on_compact_clicked}/>
        compact
    </label>
</div>

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
        width: 70%;
        position: absolute;
        bottom: 0;
        text-align: left;
    }
    #output {
        height: calc(100% - 20px);
        width: 30%;
        position: absolute;
        bottom: 0;
        right: 0;
        overflow-y: auto;
        background-color: #333;
        color: #ccc;
        white-space: pre-wrap;
    }
    #compact {
        height: 20px;
        width: 30%;
        min-width: 120px;
        position: absolute;
        right: 0;
        top: 0;
        background-color: #333;
        font-size: 14px;
        color: #ccc
    }
</style>
