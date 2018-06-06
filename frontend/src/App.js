import React, { Component } from 'react';
import './App.css';

import SearchField from './SearchField.js';
import ImageSlot from './ImageSlot.js';
import ImageEditor from './ImageEditor.js'
import TagList from './TagList.js'
import TifariAPI from "./APIComms.js"

class App extends Component {

    constructor(props) {
        super(props);

        this.state = {
            queriedImages: [],
            selectedImages: [],
            displayTagList: false,
            tagQueueSize: 0,
        };

        this.onSearch           = this.onSearch.bind(this);
        this.requeryImages      = this.requeryImages.bind(this);
        this.onSelectImage      = this.onSelectImage.bind(this);
        this.hideEditorSidebar  = this.hideEditorSidebar.bind(this);
        this.escKeyListener     = this.escKeyListener.bind(this);
        this.viewToBeTaggedList = this.viewToBeTaggedList.bind(this);
        this.mapSelectedImages  = this.mapSelectedImages.bind(this);
        this.onEditorAddTag     = this.onEditorAddTag.bind(this);
        this.onEditorRemoveTag  = this.onEditorRemoveTag.bind(this);
        this.queryTagListSize   = this.queryTagListSize.bind(this);
    }

    hideEditorSidebar() {
        this.setState({
            selectedImages: []
        });
    }

    escKeyListener(ev) {
        if(ev.key === "Escape") {
            this.hideEditorSidebar();
        }
    }

    componentWillMount() {
        this.queryTagListSize()
    }

    componentDidMount() {
        document.addEventListener("keypress", this.escKeyListener, false);
    }

    componentWillUnmount() {
        document.removeEventListener("keypress", this.escKeyListener, false);
    }

    // callback that's called when we want to select an image
    onSelectImage(img) {
        // avoid duplicate images
        for(var i = 0; i < this.state.selectedImages.length; i++) {
            if(img.id === this.state.selectedImages[i].id) {
                return;
            }
        }

        // append im to selectedImages
        this.setState(prevState => ({
          selectedImages: [...prevState.selectedImages, img]
        }));
    }

    // maps a given image array to image slots for display in tghe main page
    mapSelectedImages(images) {
        let mappedImages = images.results.map(img => 
            <ImageSlot 
                img={img} 
                key={img.id} 
                onClick={() => this.onSelectImage(img)}
            />
        );

        this.setState({queriedImages: mappedImages});
    }

    // will display search results in the main page
    requeryImages(tagsArray) {
        TifariAPI.search(tagsArray).then(this.mapSelectedImages);
    }

    // will display the to be tagged list in the main page
    viewToBeTaggedList() {
        TifariAPI.getToBeTaggedList().then(this.mapSelectedImages);
    }

    // callback that's called when we want to search the backend for tags
    onSearch(query) {
        let tags = query.split(" ");
        this.requeryImages(tags);
    }

    // callback that's called whenever we add a tag to an image
    onEditorAddTag(index, tags) {
        this.setState(oldState => {
            let newArray = oldState.selectedImages[index].tags.concat(tags);

            oldState.selectedImages[index].tags = newArray;
            return {selectedImages: oldState.selectedImages}
        });
        
        this.queryTagListSize();
    }

    // updates the tagQueueSize state from the backend
    queryTagListSize() {
        TifariAPI.getTagQueueSize()
            .then(size => {
                this.setState({
                    tagQueueSize: size
                });
            });
    }

    // callback that's called when we remove a tag from an image
    onEditorRemoveTag(image, tag) {
        TifariAPI.removeTags([tag.id], [image.id]);
        
        this.setState(oldState => {

            let imgIndex = oldState.selectedImages.findIndex(i => i.id === image.id);
            if(imgIndex === -1) return oldState;
            
            let tagIndex = oldState.selectedImages[imgIndex].tags.findIndex(t => t.id === tag.id);
            if(tagIndex === -1) return oldState;

            // remove 
            oldState.selectedImages[imgIndex].tags.splice(tagIndex, 1);

            return {selectedImages: oldState.selectedImages}
        });

        this.queryTagListSize();
    }

    render() {
        return (
            <div className="App">
                {this.state.selectedImages.length > 0 &&
                    <ImageEditor 
                        images={this.state.selectedImages}
                        addTagsToImage={this.onEditorAddTag}
                        onRemoveTag={this.onEditorRemoveTag}
                    />
                }

                {this.state.displayTagList &&
                    <TagList
                    />
                }

                <header>
                    <button onClick={this.viewToBeTaggedList}>View To-Tag List({this.state.tagQueueSize})</button>
                    <button onClick={() => TifariAPI.reloadRoot()}>Reload images</button>
            
                    <button 
                        onClick={() => this.setState({displayTagList: !this.state.displayTagList})}
                        >
                        {this.state.displayTagList ? "Hide" : "Show"} tag list
                    </button>

                    <SearchField onChange = {this.onSearch} />
                </header>

                <ul>{this.state.queriedImages}</ul>
            </div>
        );
    }
}

export default App;
