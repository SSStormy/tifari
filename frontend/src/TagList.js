import React, { Component } from 'react';
import "./TagList.css"

class TagList extends Component {
    render() {

        const tagList = this.props.tags.map(tag => 
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
