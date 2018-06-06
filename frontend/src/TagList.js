import React, { Component } from 'react';
import TifariAPI from "./APIComms.js"
import "./TagList.css"

class TagList extends Component {
    
    constructor(prop) {
        super(prop);

        this.state = {
            tags: []
        };
    }

    componentWillMount() {
        TifariAPI.getAllTags()
            .then(payload => this.setState({tags: payload}));
    }

    render() {
    
        const tagList = this.state.tags.map(tag => 
            <li key={tag.id}>{tag.name}({tag.times_used})</li>
        );

        return (
            <div className="TagList_sidebar">
                <h1>Tag List</h1>
                <ul>{tagList}</ul>
            </div>
        );
    }
}

export default TagList;
