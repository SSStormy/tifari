import React, { Component } from 'react';
import "./TagList.css"

const orderingDescTimesUsed = {
    display: "Times used, descending",
    order: function(tags) {
        tags.sort((a, b) => a.times_used < b.times_used);
    }
}

const orderingAscTimesUsed= {
    display: "Times used, ascending",
    order: function(tags) {
        tags.sort((a, b) => a.times_used > b.times_used);
    }
}

const orderingDescLexicographic = {
    display: "Alphabetical, descending",
    order: function(tags) {
        tags.sort((a, b) => a.name < b.name);
    }
}

const orderingAscLexicographic = {
    display: "Alphabetical, ascending",
    order: function(tags) {
        tags.sort((a, b) => a.name > b.name);
    }
}

const allOrderings = [
    orderingDescTimesUsed,
    orderingAscTimesUsed,
    orderingDescLexicographic,
    orderingAscLexicographic,
];

class TagList extends Component {

    render() {

        const tagList = this.props.tags.map(tag => 
            <li key={tag.id}>{tag.name}({tag.times_used})</li>
        );

        const orderingButtons = allOrderings.map(ord =>
            <div key = {ord.display}>
            <button onClick = {() => this.props.callbackSetOrdering(ord)}>
                {ord.display}
            </button>
            </div>
        );

        return (
            <div className="TagList_sidebar">
                <h1>Tag List</h1>
                <ul>{orderingButtons}</ul>
                
                <ul>{tagList}</ul>
            </div>
        );
    }
}

const defaultTagOrdering = orderingDescTimesUsed;

export { 
    TagList,
    defaultTagOrdering,
}
