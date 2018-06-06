import React, { Component } from 'react';

class SearchField extends Component {

    constructor(props) {
        super(props);

        this.state = {
            inputValue: ""
        };

        this.onInputChange = this.onInputChange.bind(this);
    }

    onInputChange(event) {
        const val = event.target.value.trim();
        this.props.onChange(val)
    }

    render() {
        return (
            <input 
                type="text"
                onChange={this.onInputChange}
            />
        );
    }
}

export default SearchField;
