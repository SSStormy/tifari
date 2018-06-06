import React, { Component } from 'react';
import './ImageEditor.css';
import TifariAPI from "./APIComms.js"

class TagRemoveButton extends Component {

    constructor(props) {
        super(props);

        this.onClicked = this.onClicked.bind(this);
    }

    onClicked() {

        // TODO : this should fire a callback in the parent to notify of removal.
        alert(this.props.tag.id);
    }

    render() {
        return (
            <button
                onClick={this.onClicked}>
                {this.props.tag.name}
            </button>
        );
    }
}

class ImageEditor extends Component {

    constructor(props) {
        super(props);

        this.state = {
            tagString: "",
        }

        this.onTagAddInputChange = this.onTagAddInputChange.bind(this);
        this.submitTags = this.submitTags.bind(this);
    }

    submitTags() {
        let tagsArray = this.state.tagString.trim().split(" ");
        this.setState({tagString: ""});

        if(0 >= tagsArray.length) { 
            return;
        }

        for(var i = 0; i < this.props.images.length; i++) {
            TifariAPI.addTags(this.props.images[i], tagsArray)
                .then(tags => {
                    // TODO : this could mutate the image array. Callback?
                    this.props.images[i].tags.concat(tags);
                });
        }
    }

    onTagAddInputChange(ev) {
        this.setState({tagString: ev.target.value.trim()});
        if(ev.key === "Enter") {
            this.submitTags();;
        }
    }

    render() {
        
        let tags = [];
        let existingTags = new Set();
        this.props.images.forEach(img => {

            img.tags.forEach(tag => {
                if(existingTags.has(tag.id)) return;

                existingTags.add(tag.id);
                tags.push(<TagRemoveButton key={tag.id} tag={tag}/>);
            })
        })

        const imgs = this.props.images.map(img => 
            <img
                key = {img.id}
                alt = {img.path}
                src = {TifariAPI.getImageUrl(img)}
            />
        );

        return (
            <div className="ImageEditor_sidebar">
                <ul>{imgs}</ul>

                <ul>{tags}</ul>

                <div>
                    <input type="text" 
                        onChange={this.onTagAddInputChange}
                    />
                                    
                    {this.state.tagString.length > 0 &&
                        <button onClick={this.submitTags}>Add tags</button>
                    }
                </div>
            </div>
        );
    }
}

export default ImageEditor;
