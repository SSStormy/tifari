import React, { Component } from 'react';

class SearchField extends Component {

    constructor(props) {
        super(props);

        this.state = {
            inputValue: ""
        };

        this.onInputChange = this.onInputChange.bind(this);
        this.onFocus= this.onFocus.bind(this);
    }

    onInputChange(event) {
        const val = event.target.value.trim();
        this.props.onChange(val)
    }
    
    onFocus(ev) {
        this.searchBar.focus();
    }

    componentDidMount() {
        this.searchBar.focus();

        document.addEventListener("focus", this.onFocus, false);
    }

    componentWillUnmount() {
        document.removeEventListener("focus", this.onFocus, false);
    }

    render() {
        return (
            <input 
                ref={(obj) => this.searchBar = obj}
                type="text"
                onChange={this.onInputChange}
            />
        );
    }
}

export default SearchField;
