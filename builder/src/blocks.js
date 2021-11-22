export default function load_blocks(Blockly) {
    const block_mixin = {
        init() {
            this.inputCounter = 1, // used to give each input a unique name
            this.appendValueInput("arg_0").appendField(this.name)
            this.setHelpUrl(`https://github.com/ylxdzsw/sopipe/tree/master/components/${this.comp_name || this.name}`)
            this.tooltip && this.setTooltip(this.tooltip)
            this.setColour(this.color)
            this.setNextStatement(true)
            this.setPreviousStatement(true)
        },

        mutationToDom() {
            const container = Blockly.utils.xml.createElement('mutation')
            const inputNames = this.inputList.map(input => input.name).join(',')
            container.setAttribute('inputNames', inputNames)
            container.setAttribute('inputCounter', this.inputCounter)
            return container
        },

        domToMutation(xmlElement) {
            const items = xmlElement.getAttribute('inputNames')
            if (items) {
                const inputNames = items.split(',')
                this.inputList = []
                inputNames.forEach((name) => this.appendValueInput(name))
                this.inputList[0].appendField(this.name)
            }
            this.inputCounter = parseInt(xmlElement.getAttribute('inputCounter'))
        },

        getIndexForNewInput(connection) {
            if (!connection.targetConnection) {
                return null
            }

            let connectionIndex
            for (let i = 0; i < this.inputList.length; i++) {
                if (this.inputList[i].connection == connection) {
                    connectionIndex = i
                }
            }

            if (connectionIndex == this.inputList.length - 1) {
                return this.inputList.length + 1
            }

            const nextInput = this.inputList[connectionIndex + 1]
            const nextConnection = nextInput && nextInput.connection.targetConnection
            if (nextConnection && !nextConnection.sourceBlock_.isInsertionMarker()) {
                return connectionIndex + 1
            }

            return null
        },

        onPendingConnection(connection) {
            const insertIndex = this.getIndexForNewInput(connection)
            if (insertIndex == null) {
                return
            }
            this.appendValueInput(`arg_${this.inputCounter++}`)
            this.moveNumberedInputBefore(this.inputList.length - 1, insertIndex)
        },

        finalizeConnections() {
            if (this.inputList.length > 1) {
                this.inputList
                    .slice(1)
                    .filter(input => !input.connection.targetConnection)
                    .map(input => this.removeInput(input.name))
            }
        },
    }

    Blockly.Blocks['tcp'] = {
        name: 'tcp',
        color: 160,
        ...block_mixin
    }

    Blockly.Blocks['xor'] = {
        name: 'xor',
        color: 120,
        ...block_mixin
    }

    Blockly.Blocks['argument'] = {
        init() {
            this.setColour(20)
            this.setOutput(true)
            this.setTooltip('Set an argument.')
            this.appendDummyInput()
                .appendField(new Blockly.FieldTextInput("", s => s.replace(/\s/g, '')), "key") // TODO: real sanitizing
                .appendField("=")
                .appendField(new Blockly.FieldTextInput(""), "value")
        }
    }
}
