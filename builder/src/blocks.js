export default function load_blocks(Blockly) {
    const dynamic_inputs_mixin = (special_names=[]) => ({
        inputCounter: 0, // used to give each input a unique name
        minInputs: 1 + special_names.length,

        init_dynamic_inputs() {
            if (special_names.length) {
                for (const special_name in special_names) {
                    this.appendValueInput(special_name)
                        .appendField(special_name)
                }
            } else {
                this.appendValueInput(".")
            }
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
                for (let i = 0; i < special_names.length; i++) {
                    if (i < this.inputList.length) {
                        this.inputList[i].appendField(special_names[i])
                    }
                }
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
            this.appendValueInput('.' + (this.inputCounter++)) // TODO: add a editable field to specify the name?
            this.moveNumberedInputBefore(this.inputList.length - 1, insertIndex)
        },

        finalizeConnections() {
            if (this.inputList.length > this.minInputs) {
                this.inputList
                    .slice(this.minInputs)
                    .filter(input => !input.connection.targetConnection)
                    .map(input => this.removeInput(input.name))
            }
        },
    })

    Blockly.Blocks['tcp'] = {
        init() {
            this.setOutput(true)
            this.setColour(160)
            this.setTooltip('tcp.')
            this.setHelpUrl('https://github.com/ylxdzsw/sopipe/tree/master/components/tcp')

            this.appendDummyInput().appendField("Tcp")
            this.appendValueInput('.next').appendField(".next")

        }
    }

    Blockly.Blocks['sucks'] = {
        init() {
            this.init_dynamic_inputs()
        },
        ...dynamic_inputs_mixin()
    }
}
