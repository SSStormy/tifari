import React, { Component } from 'react';
import './App.css';

import ImageEditor from './ImageEditor.js';
import {defaultTagOrdering} from './TagList.js';
import TifariAPI from "./APIComms.js";
import {ldebug, assert} from "./Logging.js";

import CssBaseline from '@material-ui/core/CssBaseline';
import IconButton from '@material-ui/core/IconButton';
import Tabs from '@material-ui/core/Tabs';
import Tab from '@material-ui/core/Tab';
import Icon from '@material-ui/core/Icon';
import Snackbar from '@material-ui/core/Snackbar';
import Grid from '@material-ui/core/Grid';
import Button from '@material-ui/core/Button';
import AppBar from '@material-ui/core/AppBar';
import Toolbar from '@material-ui/core/Toolbar';
import TextField from '@material-ui/core/TextField';
import Card from '@material-ui/core/Card';
import CardMedia from '@material-ui/core/CardMedia';
import CardContent from '@material-ui/core/CardContent';
import CardActions from '@material-ui/core/CardActions';
import Typography from '@material-ui/core/Typography';
import Paper from '@material-ui/core/Paper';
import GridListTile from '@material-ui/core/GridListTile';
import GridList from '@material-ui/core/GridList';
import ButtonBase from '@material-ui/core/ButtonBase';
import Drawer from '@material-ui/core/Drawer';
import Chip from '@material-ui/core/Chip';

class TagList extends Component {

    constructor(props) {
        super(props);

        this.state = {
            textField: "",
        };
    }

    submitTabInput() {
        this.props.onAdd(this.state.textField);
        this.setState({textField: ""});
    }

    render() {

        const tagList = this.props.tags.map(tag => 
            <Chip 
                key={tag.id} 
                label={tag.name}
                onDelete={() => this.props.onRemove(tag)}
            />
        );

        return (
            <div className={`${this.props.className} tag-list`}>
                {tagList}

                <span className="input-field">
                    <TextField
                        value={this.state.textField}
                        id="tag-input"
                        label="Add tags"
                        style={{paddingRight: 8}}
                        onChange={(e) => this.setState({textField: e.target.value})}
                        type="text"
                    />
                </span>

                <Button 
                    variant="fab" 
                    className="add-button"
                    onClick={() => this.submitTabInput()}
                    >
                    <Icon>add</Icon>
                </Button>
            </div>
        );
    }
}

class StateMutator {
    constructor(app, oldState) {
        this.newState = {};

        this.oldState = oldState;
        this.app = app;
    }

    getFinalState() {
        return this.newState;
    }

    getOldState() {
        return this.oldState;
    }

    setTags(tags) {
        ldebug("Setting tags list to");
        ldebug(tags);

        this.newState.tags = tags;
        return this;
    }

    setTagOrdering(ordering) {
        this.newState.tagOrdering = ordering;
        return this;
    }

    showSnackbar(msg) {
        this.newState.showSnackbar = true;
        this.newState.snackbarMessage = msg;

        return this;
    }

    hideSnackbar() {
        this.newState.showSnackbar = false;

        return this;
    }

    orderTags() {
        
        if(!this.newState.hasOwnProperty("tags")) {
            this.newState.tags = this.oldState.tags;
        }

        let tagOrdering = this.newState.hasOwnProperty("tagOrdering")
            ? this.newState.tagOrdering
            : this.oldState.tagOrdering;

        tagOrdering.order(this.newState.tags);
    }

    getProp(name) {
        return this.newState.hasOwnProperty(name)
                ? this.newState[name]
                : this.oldState[name];
    }

    getPropMarkDirty(name) {
        if(!this.newState.hasOwnProperty(name))
            this.newState[name] = this.oldState[name];

        return this.newState[name];
    }

    setActiveImageList(imgsEnum) {
        ldebug("Setting active image list to");
        ldebug(imgsEnum);
        this.newState.activeImageListEnum = imgsEnum;
        return this;
    }

    setSelectedImages(images) {
        this.newState.selectedImages = images;
        return this;
    }

    setSearchImages(images) {
        this.newState.searchImages= images;
        return this;
    }

    setToBeTaggedImages(images) {
        this.newState.toBeTaggedImages = images;
        return this;
    }

    clearImgList(imgEnum) {
        this.newState[imgEnum.prop] = [];
        return this;
    }

    addImageToList(imgsEnum, image) {
        let images = this.getPropMarkDirty(imgsEnum.prop);

        // avoid duplicate images
        if(images.findIndex(i => i.id === image.id) !== -1)
            return this;
    
        images.push(image);

        return this;
    }

    // doesn't update the image list.
    removeImageFromList(imgsEnum, image) {
        let images = this.getPropMarkDirty(imgsEnum.prop);
    
        let imgIndex = images.findIndex(i => i.id === image.id);
        if(imgIndex === -1) return this;

        images.splice(imgIndex, 1);

        return this;
    }

    // doesn't update the image list.
    removeTagFromImage(image, tag) {
        ldebug("Removing tag from image");
        ldebug(image);
        ldebug(tag);

        let tagIndex = image.tags.findIndex(t => t.id === tag.id);
        if(tagIndex === -1) return this;

        image.tags.splice(tagIndex, 1);

        return this;
    }

    // doesn't update the image list.
    // image must be a part of the app state
    addTagToImage(image, tag) {
        ldebug("Adding tag to image");
        ldebug(image);
        ldebug(tag);

        let tagIndex = image.tags.findIndex(t => t.id === tag.id);

        if(tagIndex === -1) {
            image.tags.push(tag);
        }

        return this;
    }

    setTabState(state) {
        this.newState.tabState = state;

        return this;
    }

    setSearchTags(query, tagsArray) {
        ldebug("Setting search tags");
        ldebug(query);
        ldebug(tagsArray);
        this.newState.searchInput = query;
        this.newState.searchTagNameSet = new Set(tagsArray);
        return this;
    }

    setTagListDisplayState(state) {
        ldebug("Setting tag list display state to");
        ldebug(state);

        this.newState.displayTagList = state;
        return this;
    }

    setToBeTaggedListSize(size) {
        ldebug("Seting to be tagged list size to");
        ldebug(size);

        this.newState.tagQueueSize = size;
        return this;
    }
}

const IMGS_SEARCH   = {id: 0, prop: "searchImages" }
const IMGS_TO_TAG   = {id: 1, prop: "toBeTaggedImages" }
const IMGS_SELECTED = {id: 2, prop: "selectedImages" }

const TABS_SEARCH = 0;
const TABS_TO_TAG = 1;
const TABS_SELECTED = 2;

class App extends Component {

    constructor(props) {
        super(props);

        this.state = {
            activeImageListEnum: IMGS_SEARCH,
            searchImages: [],
            selectedImages: [],
            toBeTaggedImages: [],

            displayTagList: false,
            tagQueueSize: 0,
            searchTagNameSet: new Set(),
            searchInput: "",
            tags: [],
            tagOrdering: defaultTagOrdering,
            tabState: 0,
        };

        this.foreignShowSearchTab = this.foreignShowSearchTab.bind(this);
        this.foreignShowToBeTaggedTab = this.foreignShowToBeTaggedTab.bind(this);
        this.foreignShowSelectedTab = this.foreignShowSelectedTab.bind(this);
        this.foreignSetTagListOrdering      = this.foreignSetTagListOrdering.bind(this);
        this.foreignToggleTagListDisplay    = this.foreignToggleTagListDisplay.bind(this);
        this.foreignEscKeyListener          = this.foreignEscKeyListener.bind(this);
        this.foreignAddTagToSearch = this.foreignAddTagToSearch.bind(this);
    }

    hideEditorSidebar() {
        this.mutateState(mut => mut.clearSelectedImages());
    }

    foreignEscKeyListener(ev) {
        if(ev.key === "Escape") {
            this.hideEditorSidebar();
        }
    }

    componentWillMount() {
        this.updateToBeTaggedListSize();
        this.updateTagList();
        this.updateToBeTaggedList();
    }

    updateToBeTaggedList() {
        TifariAPI.getToBeTaggedList()
            .then(images =>
                this.mutateState(mut => mut.setToBeTaggedImages(images))
            );
    }

    componentDidMount() {
        document.addEventListener("keypress", this.foreignEscKeyListener, false);
    }

    componentWillUnmount() {
        document.removeEventListener("keypress", this.foreignEscKeyListener, false);
    }

    mutateState(lambda) {
        this.setState(oldState => {
            let mutator = new StateMutator(this, oldState);
            lambda(mutator);
            return mutator.getFinalState();
        });
    }

    selectImage(img) {
        this.mutateState(mut => {
            mut.addImageToList(IMGS_SELECTED, img);
        })
    }

    // callback that's called when we want to search the backend for tags
    doImageSearch(query) {
        let tags = query.trim().split(" ");

        TifariAPI.search(tags)
            .then(images =>
                this.mutateState(mut =>
                    mut.setSearchTags(query, tags)
                       .setSearchImages(images)
                ));
    }

    foreignShowSearchTab() { 
        this.mutateState(mut => mut.setActiveImageList(IMGS_SEARCH));
    }

    foreignShowToBeTaggedTab() { 
        this.mutateState(mut => mut.setActiveImageList(IMGS_TO_TAG));
    }

    foreignShowSelectedTab() { 
        this.mutateState(mut => mut.setActiveImageList(IMGS_SELECTED));
    }

    updateToBeTaggedListSize() {
        TifariAPI.getTagQueueSize().then(size => this.mutateState(mut => mut.setToBeTaggedListSize(size)));
    }

    foreignSetTagListOrdering(ordering) {
        this.mutateState(mut => 
            mut.setTagOrdering(ordering)
               .orderTags()
        );
    }

    updateTagList() {
        TifariAPI.getAllTags().then(tags => { 
            tags.sort((a, b) => a.times_used < b.times_used);
            this.mutateState(mut => 
                mut.setTags(tags)
                   .orderTags()
            )
        });
    }

    doTagsMatchSearch(tags) {
        let searchTags= this.state.searchTagNameSet;
        
        if(searchTags.size <= 0)
            return false;

        let imageTags = new Set(tags.map(t => t.name));
    
        for(let item of searchTags) {
            if(!imageTags.has(item))
                return false;
        }

        return true;
    }

    // callback that's called when we remove a tag from an image
    removeTagFromSelected(tag) {

        let imageIds = this.state.selectedImages.map(img => img.id);

        TifariAPI.removeTags([tag.id], imageIds);

        this.mutateState(mut => {

            mut.getOldState().selectedImages
                .forEach(image => this.localBookkeepTagRemoval(mut, image, tag));
        });

        this.updateTagList();
        this.updateToBeTaggedListSize();
    }
    
    // callback that's called whenever we add a tag to an image
    addTagsToSelected(tagString) { 
        let tagNames = tagString.trim().split(" ");
        let imageIds = this.state.selectedImages.map(img => img.id);

        TifariAPI.addTags(tagNames, imageIds)
            .then(tags => this.mutateState(mut => 
                this.localBookkeepTagsAdd(mut, mut.getOldState().selectedImages, tags)
            )
        );

        this.updateTagList();
        this.updateToBeTaggedListSize();
    }

    foreignToggleTagListDisplay() {
        this.mutateState(mut => mut.setTagListDisplayState(!mut.getOldState().displayTagList));
    }

    foreignAddTagToSearch(tag) {

        if(this.state.searchTagNames.has(tag.name))
            return;

        let search = this.refSearchBar.current;
        search.value = search.value.trim();

        if(0 >= search.value.length) {
            search.value = search.value.concat(tag.name);
        } else {
            search.value = search.value.concat(" ", tag.name);
        }

        this.doImageSearch(search.value);
    }

    unselectImage(image) {
        this.mutateState(mut => { 
            mut.removeImageFromList(IMGS_SELECTED, image)
            if(mut.getFinalState()[IMGS_SELECTED.prop].length <= 0 
                && mut.getOldState().tabState === TABS_SELECTED) {
                mut.setTabState(TABS_SEARCH);
            }
        });
    }

    isImageSelected(image) {
        return -1 !== this.state.selectedImages.findIndex(i => i.id === image.id);
    }

    isViewingSelectedImages() {
        return this.state.activeImageListEnum.id == IMGS_SELECTED.id;
    }
    
    removeTagFrom(image, tag) {
        TifariAPI.removeTags([tag.id], [image.id]);

        this.mutateState(mut => {
            this.localBookkeepTagRemoval(mut, image, tag);
        });

        this.updateTagList();
        this.updateToBeTaggedListSize();

    }

    addTagsTo(image, tagString) {
        let tagNames = tagString.trim().split(" ");

        TifariAPI.addTags(tagNames, [image.id])
            .then(tags => this.mutateState(mut => 
                this.localBookkeepTagsAdd(mut, [image], tags)
            )
        );

        this.updateTagList();
        this.updateToBeTaggedListSize();

    }

    localBookkeepTagsAdd(mut, images, tags) {

        // add each tag to each image
        tags.forEach(tag => images.forEach(
                img => mut.addTagToImage(img, tag))
        );

        images.forEach(img => { 
            if(img.tags.length > 0)
                mut.removeImageFromList(IMGS_TO_TAG, img);

            if(this.doTagsMatchSearch(img.tags)) {
                mut.addImageToList(IMGS_SEARCH, img);
            }
        });
    }

    localBookkeepTagRemoval(mut, image, tag) {
        mut.removeTagFromImage(image, tag);

        if(0 >= image.tags.length) {
            mut.addImageToList(IMGS_TO_TAG, image);
            mut.removeImageFromList(IMGS_SEARCH, image);
        } else if(!this.doTagsMatchSearch(image.tags)) {
            mut.removeImageFromList(IMGS_SEARCH, image);
        }
    }

    onClickedImageCard(ev, img, isSel) {
        if(ev.target.id === "card-image") {
            if(isSel)
                this.unselectImage(img);
            else
                this.selectImage(img)
        }
    }

    render() {

        ldebug("Rendering");

        const activeImageList = this.state[this.state.activeImageListEnum.prop];
        const imageList = activeImageList.map(img => {

            const isSelected = this.isImageSelected(img);
            const drawSelectedMods = isSelected && !this.isViewingSelectedImages();

            return (
                <Grid item xs={6} key={img.id}>

                    <Card 
                        square={true} 
                        elevation={5} 
                        className="card show-when-hovering"
                        onClick={(ev) => this.onClickedImageCard(ev, img, isSelected)}
                        >

                        <img style={{opacity: drawSelectedMods ? 0.5 : 1}}
                            id="card-image"
                            src={TifariAPI.getImageUrl(img)}
                            title={img.path}
                        />

                        { drawSelectedMods &&
                        <Icon 
                            className="checkmark" 
                            style={{fontSize: 48}}
                            >
                            done_outline
                        </Icon>
                        }

                        <div className="bottom-bar show-when-hovering--on">
                            <Paper square={true} className="paper">
                                <TagList 
                                    tags={img.tags} 
                                    onAdd={(tagString) => this.addTagsTo(img, tagString)}
                                    onRemove={(tag) => this.removeTagFrom(img, tag)}
                                />
                            </Paper>
                        </div>
                        
                    </Card>


                </Grid>
            );
        });

        let selectedImageTags = [];
        if(this.state.tabState === TABS_SELECTED) {
            let existingTagIds = new Set();

            this.state.selectedImages.forEach(img => {
                img.tags.forEach(tag => {
                    if(existingTagIds.has(tag.id))
                        return;

                    existingTagIds.add(tag.id);
                    selectedImageTags.push(tag);
                })
            })
        }

        return (
            <React.Fragment>
            <CssBaseline/>
                
                <Paper className="top-bar">
                    
                    <Tabs className="center-field"
                        value={this.state.tabState} 
                        onChange={(e, v) => this.mutateState(mut => mut.setTabState(v))}
                        >

                        <Tab
                            label="Search" 
                            onClick={this.foreignShowSearchTab}
                        />
                        
                        { this.state.tagQueueSize > 0 &&
                        <Tab 
                            label={`To-be tagged (${this.state.tagQueueSize})`} 
                            onClick={this.foreignShowToBeTaggedTab}
                        />
                        }
                        
                        { this.state.selectedImages.length > 0 &&
                        <Tab 
                            label={`Selected (${this.state.selectedImages.length})`} 
                            onClick={this.foreignShowSelectedTab}
                        />
                        }

                    </Tabs>

                    { this.state.tabState === TABS_SEARCH &&
                        <TextField 
                            className="center-field"
                            autoFocus = {true}
                            label = "Search by tags"
                            type = "text"
                            value = {this.state.searchInput}
                            onChange = {ev => this.doImageSearch(ev.target.value)}
                        />
                    }

                    { this.state.tabState === TABS_SELECTED &&
                        <TagList 
                            className="center-field"
                            tags={selectedImageTags}
                            onAdd={(tagString) => this.addTagsToSelected(tagString)}
                            onRemove={(tag) => this.removeTagFromSelected(tag)}
                        />
                    }
                </Paper>

                <div className="image-list">
                    <Grid container spacing={16}>
                        {imageList}
                    </Grid>
                </div>

                <Snackbar
                    open={this.state.showSnackbar}
                    anchorOrigin={{ vertical: "bottom", horizontal: "left" }}
                    onClose={() => this.mutateState(mut => mut.hideSnackbar())}
                    ContentProps={{
                        'aria-describedby': 'message-id',
                    }}
                    message={<span id="message-id">{this.state.snackbarMessage}</span>}
                />

            </React.Fragment>
        );
    }
}
/*

                {this.state.displayTagList &&
                    <TagList 
                        tags = {this.state.tags}
                        callbackSetOrdering = {this.foreignSetTagListOrdering}
                        callbackAddTag = {this.foreignAddTagToSearch}
                    />
                }

                {this.state.selectedImages.length > 0 &&
                    <ImageEditor 
                        images = {this.state.selectedImages}
                        onAddTag = {this.foreignOnEditorAddTagToSelected}
                        onRemoveTag = {this.foreignOnEditorRemoveTagFromSelected}
                        callbackRemoveImageFromSelected = {this.foreignRemoveImageFromSelected}
                    />
                }



                    <div className="search-field">

                        <IconButton 
                            className="bar-icon"
                            color="inherit"
                            aria-label="open drawer"
                            onClick={() => {}}
                            >

                            <Icon>menu</Icon>
                        </IconButton>



                    </div>

                </Paper>
    


                         <Button onClick={this. onClick=foreignViewToBeTaggedList}>
                            To-Tag List({this.state.tagQueueSize})
                        </Button>
                        <Button 
                            onClick={() => TifariAPI.reloadRoot().then(
                                () => this.mutateState(mut => mut.showSnackbar("Reloaded images")))}
                            >
                            Reload Images
                        </Button>

                        <Button onClick={this.foreignToggleTagListDisplay}>
                            {this.state.displayTagList ? "Hide" : "Show"} tag list
                        </Button>

 */

export default App;
