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
        };

        this.onSearch = this.onSearch.bind(this);
        this.requeryImages = this.requeryImages.bind(this);
        this.onSelectImage= this.onSelectImage.bind(this);
        this.hideEditorSidebar= this.hideEditorSidebar.bind(this);
        this.escKeyListener = this.escKeyListener.bind(this);
        this.viewToBeTaggedList = this.viewToBeTaggedList.bind(this);
        this.mapSelectedImages = this.mapSelectedImages.bind(this);
        this.onEditorAddTag= this.onEditorAddTag.bind(this);
        this.onEditorRemoveTag = this.onEditorRemoveTag.bind(this);
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

    componentDidMount() {
        document.addEventListener("keypress", this.escKeyListener, false);
    }

    componentWillUnmount() {
        document.removeEventListener("keypress", this.escKeyListener, false);
    }

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

    requeryImages(tagsArray) {
        TifariAPI.search(tagsArray).then(this.mapSelectedImages);
    }

    viewToBeTaggedList() {
        TifariAPI.getToBeTaggedList().then(this.mapSelectedImages);
    }

    onSearch(query) {
        let tags = query.split(" ");
        this.requeryImages(tags);
    }

    onEditorAddTag(index, tags) {
        this.setState(oldState => {
            let newArray = oldState.selectedImages[index].tags.concat(tags);

            oldState.selectedImages[index].tags = newArray;
            return {selectedImages: oldState.selectedImages}
        });
    }

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
        })
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
                    <button onClick={this.viewToBeTaggedList}>View To-Tag List</button>
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
