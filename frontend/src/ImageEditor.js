import React, { Component } from 'react';
import './ImageEditor.css';
import TifariAPI from "./APIComms.js"

class ImageEditor extends Component {

    constructor(props) {
        super(props);

        this.state = {
            tagString: "",
        };

        this.tagInputField = React.createRef();

        this.foreignOnTagAddInputChange = this.foreignOnTagAddInputChange.bind(this);
    }

    submitTags() {
        let tagsArray = this.state.tagString.trim().split(" ");
        this.setState({tagString: ""});

        // clear tag input field
        this.tagInputField.current.value = "";

        if(0 >= tagsArray.length) { 
            return;
        }

        this.props.onAddTag(tagsArray);
    }

    foreignOnTagAddInputChange(ev) {
        this.setState({tagString: ev.target.value.trim()});
        if(ev.key === "Enter") {
            this.submitTags();
        }
    }

    render() {
        let tags = [];
        let existingTags = new Set();
        this.props.images.forEach(img => {

            img.tags.forEach(tag => {
                if(existingTags.has(tag.id)) return;

                existingTags.add(tag.id);
                tags.push(
                    <button 
                        key = {tag.id}
                        onClick={() => this.props.onRemoveTag(tag)}
                        >

                        {tag.name}
                    </button>
                );
            })
        });

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
                        ref = {this.tagInputField}
                        onChange = {this.foreignOnTagAddInputChange}
                    />
                                    
                    {this.state.tagString.length > 0 &&
                        <button onClick = {() => this.submitTags()}>
                            Add tags
                        </button>
                    }
                </div>
            </div>
        );
    }
}

export default ImageEditor;
